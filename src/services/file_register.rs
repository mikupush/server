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

use crate::domain::FileUpload;
use crate::repository::FileUploadRepository;
use crate::routes::FileCreate;
use crate::services::{FileSizeLimiter, FileUploadError};
use chrono::Utc;

#[derive(Debug, Clone)]
pub struct FileRegister<FR>
where
    FR: FileUploadRepository + Clone,
{
    repository: FR,
    limiter: FileSizeLimiter
}

impl<FR> FileRegister<FR>
where
    FR: FileUploadRepository + Clone,
{
    pub fn new(repository: FR, limiter: FileSizeLimiter) -> Self {
        Self { repository, limiter }
    }

    pub fn register_file(&self, file_create: FileCreate) -> Result<(), FileUploadError> {
        if self.limiter.check_file_size(file_create.size as u64) == false {
            return Err(FileUploadError::MaxFileSizeExceeded)
        }

        let file_upload = FileUpload {
            id: file_create.id,
            name: file_create.name,
            mime_type: file_create.mime_type,
            size: file_create.size,
            uploaded_at: Utc::now().naive_utc(),
            chunked: false
        };

        let existing = self.repository.find_by_id(file_create.id)?;

        if existing.is_some() {
            return Err(FileUploadError::Exists)
        }

        self.repository.save(file_upload)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::database::tests::get_test_database_connection;
    use crate::repository::PostgresFileUploadRepository;
    use crate::services::{FileRegister, FileSizeLimiter};

    impl FileRegister<PostgresFileUploadRepository> {
        pub fn test() -> Self {
            let pool = get_test_database_connection();
            Self::new(
                PostgresFileUploadRepository::new(pool),
                FileSizeLimiter::create()
            )
        }
    }
}
