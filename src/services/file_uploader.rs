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

use std::fs::{File, OpenOptions};
use std::io::Write;
use crate::errors::FileUploadError;
use crate::repository::FileUploadRepository;
use crate::services::FileSizeLimiter;
use actix_web::web::Payload;
use futures::StreamExt;
use tracing::debug;
use uuid::Uuid;
use crate::config::Settings;

#[derive(Debug, Clone)]
pub struct FileUploader<FR>
where
    FR: FileUploadRepository + Clone,
{
    repository: FR,
    settings: Settings,
    limiter: FileSizeLimiter
}

impl<FR> FileUploader<FR>
where
    FR: FileUploadRepository + Clone,
{
    pub fn new(repository: FR, settings: Settings, limiter: FileSizeLimiter) -> Self {
        Self { repository, settings, limiter }
    }

    pub async fn upload_file(&self, id: Uuid, mut payload: Payload) -> Result<(), FileUploadError> {
        let mut total_bytes: u64 = 0;
        let file_upload = match self.repository.find_by_id(id)? {
            Some(file_upload) => file_upload,
            None => {
                debug!("file upload {} does not exist", id);
                return Err(FileUploadError::NotExists { id })
            }
        };

        let destination_path = file_upload.directory(&self.settings)?;
        let destination_path = destination_path.join(file_upload.name.clone());

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
                self.limiter.check_file_size(total_bytes);
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


#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::database::DbPool;
    use crate::repository::PostgresFileUploadRepository;
    use crate::services::{FileSizeLimiter, FileUploader};

    impl FileUploader<PostgresFileUploadRepository> {
        pub fn test(pool: DbPool) -> Self {
            Self {
                repository: PostgresFileUploadRepository::new(pool),
                settings: Settings::default(),
                limiter: FileSizeLimiter::create()
            }
        }
    }
}
