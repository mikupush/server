use crate::config::Settings;
use crate::domain::{Manifest, Part};
use rusqlite::types::FromSqlError;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, PartialEq)]
enum ManifestError {
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

trait ManifestRepository {
    fn find_by_upload_id(&self, upload_id: Uuid) -> Result<Manifest, ManifestError>;
    fn put_part(&self, part: Part) -> Result<(), ManifestError>;
}

#[derive(Debug, Clone)]
struct SQLiteManifestRepository {
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
        self.setup_manifest_schema(&connection)?;
        Ok(connection)
    }

    fn setup_manifest_schema(&self, connection: &Connection) -> Result<(), ManifestError> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS `parts` (
                `id` TEXT PRIMARY KEY,
                `index` INTEGER NOT NULL,
                `upload_id` TEXT NOT NULL
            );
        "#;

        connection.execute(sql, [])?;
        Ok(())
    }

    fn map_part(row: &rusqlite::Row) -> rusqlite::Result<Part> {
        Ok(Part {
            id: Uuid::parse_str(row.get::<usize, String>(0)?.as_str())
                .map_err(|err| FromSqlError::Other(err.into()))?,
            index: row.get(1)?,
            upload_id: Uuid::parse_str(row.get::<usize, String>(2)?.as_str())
                .map_err(|err| FromSqlError::Other(err.into()))?
        })
    }
}

impl ManifestRepository for SQLiteManifestRepository {
    fn find_by_upload_id(&self, upload_id: Uuid) -> Result<Manifest, ManifestError> {
        let connection = self.create_connection(upload_id)?;
        let mut stmt = connection.prepare(r#"
            SELECT `id`, `index`, `upload_id`
            FROM `parts` WHERE `upload_id` = ?1
        "#)?;
        let result = stmt.query_map(&[&upload_id.to_string()], Self::map_part)?;

        Ok(Manifest {
            upload_id,
            parts: result
                .filter(|item| item.is_ok())
                .map(|item| item.unwrap())
                .collect()
        })
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
            r#"
                INSERT INTO `parts` (`id`, `index`, `upload_id`)
                VALUES (?1, ?2, ?3)
            "#,
            params![
                part.id.to_string(),
                part.index,
                part.upload_id.to_string()
            ]
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
}
