use std::path::Path;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use uuid::Uuid;
use crate::config::Settings;
use crate::database::DbPool;
use crate::errors::FileDeleteError;
use crate::model::FileUpload;
use crate::schema::file_uploads;

#[derive(Debug, Clone)]
pub struct FileDeleter {
    pool: DbPool,
    settings: Settings,
}

impl FileDeleter {
    pub fn new(pool: DbPool, settings: Settings) -> Self {
        Self { pool, settings }
    }

    pub fn delete(&self, id: Uuid) -> Result<(), FileDeleteError> {
        let mut connection = self.pool.get()?;
        let file_upload: Option<FileUpload> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()?;

        let Some(file_upload) = file_upload else {
            return Err(FileDeleteError::NotFound);
        };

        let directory = self.settings.upload.directory().clone();
        let path = Path::new(&directory).join(file_upload.name);

        if path.exists() {
            std::fs::remove_file(path)?;
        }

        diesel::delete(file_uploads::table.find(id))
            .execute(&mut connection)?;

        Ok(())
    }
}
