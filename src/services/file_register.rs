use crate::database::DbPool;
use crate::errors::FileUploadError;
use crate::model::FileUpload;
use crate::routes::FileCreate;
use crate::schema::file_uploads;
use crate::services::FileSizeLimiter;
use chrono::Utc;
use diesel::OptionalExtension;
use diesel::{QueryDsl, RunQueryDsl};

#[derive(Debug, Clone)]
pub struct FileRegister {
    pool: DbPool,
    limiter: FileSizeLimiter
}

impl FileRegister {
    pub fn new(pool: DbPool, limiter: FileSizeLimiter) -> Self {
        Self { pool, limiter }
    }

    pub fn register_file(&self, file_create: FileCreate) -> Result<(), FileUploadError> {
        self.limiter.check_file_size(file_create.size as u64)?;

        let file_upload = FileUpload {
            id: file_create.id,
            name: file_create.name,
            mime_type: file_create.mime_type,
            size: file_create.size,
            uploaded_at: Utc::now().naive_utc()
        };

        let mut connection = self.pool.get()?;
        let existing: Option<FileUpload> = file_uploads::table
            .find(file_create.id)
            .first(&mut connection)
            .optional()?;

        if existing.is_some() {
            return Err(FileUploadError::Exists)
        }

        diesel::insert_into(file_uploads::table)
            .values(&file_upload)
            .execute(&mut connection)?;

        Ok(())
    }
}
