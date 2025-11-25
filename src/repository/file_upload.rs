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
use crate::database::{get_database_connection, DbPool};
use crate::model::FileUpload;
use crate::model::FileUploadModel as FileUploadModel;
use crate::schema::file_uploads;
use diesel::result::Error as DieselError;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use r2d2::Error as PoolError;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Debug)]
pub enum FileUploadRepositoryError {
    Pool(PoolError),
    Db(DieselError),
}

impl From<PoolError> for FileUploadRepositoryError {
    fn from(value: PoolError) -> Self {
        Self::Pool(value)
    }
}

impl From<DieselError> for FileUploadRepositoryError {
    fn from(value: DieselError) -> Self {
        Self::Db(value)
    }
}

pub trait FileUploadRepository {
    fn find_by_id(&self, file_upload_id: Uuid) -> Result<Option<FileUpload>, FileUploadRepositoryError>;
    fn delete(&self, file_upload_id: Uuid) -> Result<(), FileUploadRepositoryError>;
    fn save(&self, file_upload: FileUpload) -> Result<(), FileUploadRepositoryError>;
}

#[derive(Debug, Clone)]
pub struct InMemoryFileUploadRepository {
    file_uploads: Arc<Mutex<HashMap<Uuid, FileUpload>>>
}

impl InMemoryFileUploadRepository {
    pub fn new(items: HashMap<Uuid, FileUpload>) -> Self {
        Self { file_uploads: Arc::new(Mutex::new(items)) }
    }
}

impl FileUploadRepository for InMemoryFileUploadRepository {
    fn find_by_id(&self, file_upload_id: Uuid) -> Result<Option<FileUpload>, FileUploadRepositoryError> {
        let items = self.file_uploads.lock().unwrap();
        Ok(items.get(&file_upload_id).cloned())
    }

    fn delete(&self, file_upload_id: Uuid) -> Result<(), FileUploadRepositoryError> {
        let mut items = self.file_uploads.lock().unwrap();
        items.remove(&file_upload_id);
        Ok(())
    }

    fn save(&self, file_upload: FileUpload) -> Result<(), FileUploadRepositoryError> {
        let mut items = self.file_uploads.lock().unwrap();
        items.insert(file_upload.id, file_upload);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PostgresFileUploadRepository {
    db_pool: DbPool
}

impl PostgresFileUploadRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub fn get_with_settings(settings: Settings) -> Self {
        Self::new(get_database_connection(settings))
    }
}

impl FileUploadRepository for PostgresFileUploadRepository {
    fn find_by_id(&self, file_upload_id: Uuid) -> Result<Option<FileUpload>, FileUploadRepositoryError> {
        let mut connection = self.db_pool.get()?;
        let record: Option<FileUploadModel> = file_uploads::table
            .find(file_upload_id)
            .first(&mut connection)
            .optional()?;

        Ok(record.map(FileUpload::from))
    }

    fn delete(&self, file_upload_id: Uuid) -> Result<(), FileUploadRepositoryError> {
        let mut connection = self.db_pool.get()?;
        diesel::delete(file_uploads::table.find(file_upload_id))
            .execute(&mut connection)?;

        Ok(())
    }

    fn save(&self, file_upload: FileUpload) -> Result<(), FileUploadRepositoryError> {
        let mut connection = self.db_pool.get()?;
        let model: FileUploadModel = file_upload.into();
        diesel::insert_into(file_uploads::table)
            .values(&model)
            .execute(&mut connection)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::setup_database_connection;
    use chrono::Utc;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_find_by_id() {
        let pool = setup_database_connection(&Settings::load());
        let repository = PostgresFileUploadRepository::new(pool.clone());
        let file_upload = insert_file_upload(&pool);

        let stored = repository.find_by_id(file_upload.id).unwrap();

        let stored = stored.expect("file upload should exist after save");
        assert_eq!(stored.id, file_upload.id);
        assert_eq!(stored.name, file_upload.name);
        assert_eq!(stored.mime_type, file_upload.mime_type);
        assert_eq!(stored.size, file_upload.size);
    }

    #[test]
    #[serial]
    fn test_find_by_id_not_found() {
        let pool = setup_database_connection(&Settings::load());
        let repository = PostgresFileUploadRepository::new(pool.clone());

        let result = repository.find_by_id(Uuid::new_v4()).unwrap();

        assert!(result.is_none());
    }

    #[test]
    #[serial]
    fn test_delete_file_upload() {
        let pool = setup_database_connection(&Settings::load());
        let repository = PostgresFileUploadRepository::new(pool.clone());
        let file_upload = insert_file_upload(&pool);

        repository.delete(file_upload.id).unwrap();

        let stored = find_file_upload(&pool, file_upload.id);
        assert!(stored.is_none(), "file upload should be removed from database");
    }

    #[test]
    #[serial]
    fn test_save_file_upload() {
        let pool = setup_database_connection(&Settings::load());
        let repository = PostgresFileUploadRepository::new(pool.clone());
        let file_upload: FileUpload = create_file_upload().into();

        repository.save(file_upload.clone()).unwrap();

        let stored = find_file_upload(&pool, file_upload.id);
        assert!(stored.is_some(), "file upload should be saved to database");
    }

    fn create_file_upload() -> FileUploadModel {
        FileUploadModel {
            id: Uuid::new_v4(),
            name: "hatsune_miku.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size: 1024,
            uploaded_at: Utc::now().naive_utc(),
            chunked: false
        }
    }

    fn insert_file_upload(pool: &DbPool) -> FileUpload {
        let model = create_file_upload();
        let mut connection = pool.get().unwrap();

        diesel::insert_into(file_uploads::table)
            .values(&model)
            .execute(&mut connection)
            .unwrap();

        model.into()
    }

    fn find_file_upload(pool: &DbPool, id: Uuid) -> Option<FileUploadModel> {
        let mut connection = pool.get().unwrap();

        file_uploads::table.find(id)
            .first(&mut connection)
            .optional()
            .unwrap()
    }
}
