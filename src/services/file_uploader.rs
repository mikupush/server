use std::fs::{File, OpenOptions};
use std::io::Write;
use crate::database::DbPool;
use crate::errors::FileUploadError;
use crate::schema::file_uploads;
use crate::services::FileSizeLimiter;
use actix_web::web::Payload;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use futures::StreamExt;
use log::debug;
use uuid::Uuid;
use crate::model::FileUpload;

#[derive(Debug, Clone)]
pub struct FileUploader {
    pool: DbPool,
    limiter: FileSizeLimiter
}

impl FileUploader {
    pub fn new(pool: DbPool, limiter: FileSizeLimiter) -> Self {
        Self { pool, limiter }
    }

    pub async fn upload_file(&self, id: Uuid, mut payload: Payload) -> Result<(), FileUploadError> {
        let mut total_bytes: u64 = 0;
        let mut connection = self.pool.get()?;
        let file_upload: Option<FileUpload> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()?;

        let Some(file_upload) = file_upload else {
            debug!("file upload {} does not exist", id);
            return Err(FileUploadError::NotExists { id })
        };

        let destination_path = file_upload.name.clone();

        {
            if let Ok(_) = File::open(destination_path.clone()) {
                debug!("file {} exists, deleting it", file_upload.name.clone());
                std::fs::remove_file(destination_path.clone())?;
            }
        }

        {
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(destination_path.clone())?;

            while let Some(chunk) = payload.next().await {
                let bytes = match chunk {
                    Ok(bytes) => bytes,
                    Err(error) => return Err(error.into())
                };

                total_bytes += bytes.len() as u64;
                self.limiter.check_file_size(total_bytes)?;
                file.write_all(&bytes)?;
                debug!("wrote {} bytes to file {}", bytes.len(), file_upload.name);
            }
        }

        if total_bytes < file_upload.size as u64 {
            debug!("file {} upload is not completed: expected {} and given {} bytes wrote", file_upload.name, file_upload.size, total_bytes);
            std::fs::remove_file(destination_path.clone())?;
            return Err(FileUploadError::NotCompleted)
        }

        Ok(())
    }
}
