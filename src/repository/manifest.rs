// Miku Push! Server is the backend behind Miku Push!
// Copyright (C) 2025  Miku Push! Team
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
// 
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::config::Settings;
use crate::model::{Manifest, Part};
use rusqlite::types::FromSqlError;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug, PartialEq)]
pub enum ManifestError {
    IO(String),
    DuplicatedPart
}

impl From<rusqlite::Error> for ManifestError {
    fn from(err: rusqlite::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl From<std::io::Error> for ManifestError {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err.to_string())
    }
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(msg) => write!(f, "{}", msg),
            Self::DuplicatedPart => write!(f, "Duplicated part")
        }
    }
}

pub trait ManifestRepository {
    fn find_by_upload_id(&self, upload_id: Uuid) -> Result<Manifest, ManifestError>;
    fn put_part(&self, part: Part) -> Result<(), ManifestError>;
    fn take_parts(&self, upload_id: Uuid, size: i32, from_index: i32) -> Result<Vec<Part>, ManifestError>;
}

#[derive(Debug, Clone)]
pub struct InMemoryManifestRepository {
    parts: Arc<Mutex<HashMap<Uuid, Part>>>
}

impl InMemoryManifestRepository {
    pub fn new() -> Self {
        Self { parts: Arc::new(Mutex::new(HashMap::new())) }
    }
}

impl ManifestRepository for InMemoryManifestRepository {
    fn find_by_upload_id(&self, upload_id: Uuid) -> Result<Manifest, ManifestError> {
        let parts = self.parts.lock().unwrap();
        let parts = parts.values()
            .filter(|part| part.upload_id == upload_id)
            .cloned()
            .collect();

        Ok(Manifest { upload_id, parts })
    }

    fn put_part(&self, part: Part) -> Result<(), ManifestError> {
        let mut parts = self.parts.lock().unwrap();
        let existing_part = parts.get(&part.id);
        if existing_part.is_some() {
            return Err(ManifestError::DuplicatedPart);
        }

        parts.insert(part.id, part);
        Ok(())
    }

    fn take_parts(
        &self,
        upload_id: Uuid,
        size: i32,
        from_index: i32
    ) -> Result<Vec<Part>, ManifestError> {
        Ok(Vec::new())
    }
}

pub const CREATE_MANIFEST_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS `parts` (
    `id` TEXT PRIMARY KEY,
    `index` INTEGER NOT NULL,
    `upload_id` TEXT NOT NULL,
    `size` INTEGER NOT NULL
);
"#;

pub const INSERT_MANIFEST_PART_SQL: &str = r#"
INSERT INTO `parts` (`id`, `index`, `upload_id`, `size`)
VALUES (?1, ?2, ?3, ?4)
"#;

#[derive(Debug, Clone)]
pub struct SQLiteManifestRepository {
    settings: Settings
}

impl SQLiteManifestRepository {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }
}

impl SQLiteManifestRepository {
    fn create_connection(&self, upload_id: Uuid) -> Result<Connection, ManifestError> {
        let directory = PathBuf::from(self.settings.upload.directory())
            .join(upload_id.to_string());

        if !directory.exists() {
            std::fs::create_dir_all(directory.clone())?;
        }

        let path = directory.join("manifest");
        let connection = Connection::open(path)?;
        connection.execute(CREATE_MANIFEST_SCHEMA_SQL, [])?;
        Ok(connection)
    }

    fn map_part(row: &rusqlite::Row) -> rusqlite::Result<Part> {
        Ok(Part {
            id: Uuid::parse_str(row.get::<usize, String>(0)?.as_str())
                .map_err(|err| FromSqlError::Other(err.into()))?,
            index: row.get(1)?,
            upload_id: Uuid::parse_str(row.get::<usize, String>(2)?.as_str())
                .map_err(|err| FromSqlError::Other(err.into()))?,
            size: row.get(3)?,
        })
    }
}

impl ManifestRepository for SQLiteManifestRepository {
    fn find_by_upload_id(&self, upload_id: Uuid) -> Result<Manifest, ManifestError> {
        let connection = self.create_connection(upload_id)?;
        let mut stmt = connection.prepare(r#"
            SELECT `id`, `index`, `upload_id`, `size`
            FROM `parts` WHERE `upload_id` = ?1
        "#)?;
        let result = stmt.query_map(&[&upload_id.to_string()], Self::map_part)?;
        let parts: Vec<Part> = result
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect();

        Ok(Manifest { upload_id, parts })
    }

    fn put_part(&self, part: Part) -> Result<(), ManifestError> {
        let connection = self.create_connection(part.upload_id)?;
        let existing_stmt = connection.prepare(r#"
            SELECT COUNT(`id`)
            FROM `parts`
            WHERE `upload_id` = ?1 AND `index` = ?2
        "#);

        let existing: i64 = existing_stmt?.query_row(
            params![
                part.upload_id.to_string(),
                part.index
            ],
            |row| row.get(0)
        )?;

        if existing > 0 {
            return Err(ManifestError::DuplicatedPart);
        }

        connection.execute(
            INSERT_MANIFEST_PART_SQL,
            params![
                part.id.to_string(),
                part.index,
                part.upload_id.to_string(),
                part.size
            ]
        )?;

        Ok(())
    }

    fn take_parts(
        &self,
        upload_id: Uuid,
        size: i32,
        from_index: i32
    ) -> Result<Vec<Part>, ManifestError> {
        let connection = self.create_connection(upload_id)?;
        let mut parts_stmt = connection.prepare(r#"
            SELECT `id`, `index`, `upload_id`, `size`
            FROM `parts`
            WHERE `index` > ?2
            ORDER BY `index` ASC
            LIMIT ?3
        "#)?;

        let parts = parts_stmt.query_map(
            params![upload_id.to_string(), from_index, size],
            Self::map_part
        )?;

        let parts: Vec<Part> = parts
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect();

        Ok(parts)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn create_repository() -> SQLiteManifestRepository {
        let settings = Settings::load();
        SQLiteManifestRepository::new(settings)
    }

    #[test]
    fn test_put_part() {
        let repository = create_repository();
        let upload_id = Uuid::new_v4();
        

        let part = Part::new(upload_id, 0);
        let result = repository.put_part(part.clone());

        assert!(result.is_ok());
    }

    #[test]
    fn test_put_part_duplicate_error() {
        let repository = create_repository();
        let upload_id = Uuid::new_v4();

        let part = Part::new(upload_id, 0);
        repository.put_part(part.clone()).unwrap();
        let result = repository.put_part(part.clone());

        assert_eq!(result, Err(ManifestError::DuplicatedPart));
    }

    #[test]
    fn test_find_by_upload_id_empty() {
        let repository = create_repository();
        let upload_id = Uuid::new_v4();

        let manifest = repository.find_by_upload_id(upload_id).unwrap();

        assert_eq!(manifest.upload_id, upload_id);
        assert_eq!(manifest.parts.len(), 0);
    }

    #[test]
    fn test_find_by_upload_id_with_parts() {
        let repository = create_repository();
        let upload_id = Uuid::new_v4();
        
        for part in 0..3 {
            let part = Part::new(upload_id, part);
            repository.put_part(part.clone()).unwrap();
        }

        let manifest = repository.find_by_upload_id(upload_id).unwrap();

        assert_eq!(manifest.upload_id, upload_id);
        assert_eq!(manifest.parts.len(), 3);

        for part in 0..3 {
            assert_eq!(manifest.parts[part].index, part as i64);
            assert_eq!(manifest.parts[part].upload_id, upload_id);
        }
    }

    #[test]
    fn test_multiple_uploads_isolated() {
        let repository = create_repository();
        let upload_id1 = Uuid::new_v4();
        let upload_id2 = Uuid::new_v4();

        let part1 = Part::new(upload_id1, 0);
        let part2 = Part::new(upload_id2, 0);

        repository.put_part(part1.clone()).unwrap();
        repository.put_part(part2.clone()).unwrap();

        let manifest1 = repository.find_by_upload_id(upload_id1).unwrap();
        let manifest2 = repository.find_by_upload_id(upload_id2).unwrap();

        assert_eq!(manifest1.parts.len(), 1);
        assert_eq!(manifest2.parts.len(), 1);
        assert_eq!(manifest1.parts[0].id, part1.id);
        assert_eq!(manifest2.parts[0].id, part2.id);
    }

    pub fn insert_test_manifest_part(settings: &Settings, part: &Part) {
        let directory = PathBuf::from(settings.upload.directory())
            .join(part.upload_id.to_string());

        if !directory.exists() {
            std::fs::create_dir_all(directory.clone()).unwrap();
        }

        let path = directory.join("manifest");
        let connection = Connection::open(path).unwrap();
        connection.execute(CREATE_MANIFEST_SCHEMA_SQL, []).unwrap();
        connection.execute(
            INSERT_MANIFEST_PART_SQL,
            params![
                part.id.to_string(),
                part.index,
                part.upload_id.to_string(),
                part.size
            ]
        ).unwrap();
    }
}
