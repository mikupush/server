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
use crate::model::FileUpload;
use crate::repository::{FileUploadRepository, PostgresFileUploadRepository};
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

impl FileRegister<PostgresFileUploadRepository> {
    pub fn get_with_settings(settings: Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(settings.clone()),
            FileSizeLimiter::new(settings)
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::model::FileUpload;
    use crate::repository::InMemoryFileUploadRepository;
    use crate::routes::FileCreate;
    use crate::services::{FileRegister, FileSizeLimiter, FileUploadError};
    use std::collections::HashMap;
    use uuid::Uuid;

    impl FileRegister<InMemoryFileUploadRepository> {
        pub fn create() -> Self {
            Self::new(
                Self::create_repository(),
                FileSizeLimiter::create()
            )
        }

        pub fn create_limited() -> Self {
            Self::new(
                Self::create_repository(),
                FileSizeLimiter::create_limited()
            )
        }

        fn create_repository() -> InMemoryFileUploadRepository {
            let items: HashMap<Uuid, FileUpload> = HashMap::from([(
                Uuid::parse_str("9317393a-c4ef-4b69-bfb9-060050f0879a").unwrap(),
                FileUpload::create("9317393a-c4ef-4b69-bfb9-060050f0879a")
            )]);
            InMemoryFileUploadRepository::new(items)
        }
    }

    #[test]
    fn test_register_file() {
        let register = FileRegister::create();
        let id = Uuid::new_v4();
        let file_create = FileCreate {
            id,
            name: String::from("test.txt"),
            mime_type: String::from("text/plain"),
            size: 100
        };

        let result = register.register_file(file_create);
        assert!(result.is_ok());
    }

    #[test]
    fn test_register_file_exists() {
        let register = FileRegister::create();
        let id = Uuid::parse_str("9317393a-c4ef-4b69-bfb9-060050f0879a").unwrap();
        let file_create = FileCreate {
            id,
            name: String::from("test.txt"),
            mime_type: String::from("text/plain"),
            size: 100
        };

        let result = register.register_file(file_create);
        assert!(result.is_err());
        assert_eq!(FileUploadError::Exists, result.unwrap_err());
    }

    #[test]
    fn test_register_file_exceed_max_size() {
        let register = FileRegister::create_limited();
        let id = Uuid::new_v4();
        let file_create = FileCreate {
            id,
            name: String::from("test.txt"),
            mime_type: String::from("text/plain"),
            size: 300
        };

        let result = register.register_file(file_create);
        assert!(result.is_err());
        assert_eq!(FileUploadError::MaxFileSizeExceeded, result.unwrap_err());
    }
}
