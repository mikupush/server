use crate::config::Settings;
use crate::database::DbPool;
use crate::errors::FileUploadError;
use crate::model::FileUpload;
use crate::routes::FileCreate;
use crate::schema::file_uploads;
use chrono::Utc;
use diesel::OptionalExtension;
use diesel::{QueryDsl, RunQueryDsl};
use log::debug;

#[derive(Debug, Clone)]
pub struct FileUploadRegister {
    pool: DbPool,
    settings: Settings
}

impl FileUploadRegister {
    pub fn new(pool: DbPool, settings: Settings) -> Self {
        Self { pool, settings }
    }

    pub fn register_file(&self, file_create: FileCreate) -> Result<(), FileUploadError> {
        if self.settings.upload.is_limited() {
            let file_size = file_create.size as u64;
            let limit = self.settings.upload.max_size().unwrap();
            debug!("file size is limited by: {} bytes", limit);

            if file_size > limit {
                debug!("file size limit exceeded: {} > {} bytes", file_size, limit);
                return Err(FileUploadError::MaxFileSizeExceeded)
            }
        }

        let file_upload = FileUpload {
            id: file_create.id,
            name: file_create.name,
            mime_type: file_create.mime_type,
            size: file_create.size,
            uploaded_at: Utc::now().naive_utc()
        };

        let mut connection = self.pool.get()
            .map_err(|err| FileUploadError::Other { message: err.to_string() })?;

        let existing: Option<FileUpload> = file_uploads::table
            .find(file_create.id)
            .first(&mut connection)
            .optional()
            .map_err(|err| FileUploadError::Other { message: err.to_string() })?;

        if existing.is_some() {
            return Err(FileUploadError::Exists)
        }

        diesel::insert_into(file_uploads::table)
            .values(&file_upload)
            .execute(&mut connection)
            .map_err(|err| FileUploadError::Other { message: err.to_string() })?;

        Ok(())
    }
}
