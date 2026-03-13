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
use crate::services::{FileSizeLimiter, FileUploadError, SystemClock};
use chrono::Duration;
use crate::cache::MokaCache;
use crate::clock::Clock;

#[derive(Debug, Clone)]
pub struct FileRegister<FR, C>
where
    FR: FileUploadRepository + Clone,
    C: Clock + Clone
{
    repository: FR,
    limiter: FileSizeLimiter,
    settings: Settings,
    clock: C,
}

impl<FR, C> FileRegister<FR, C>
where
    FR: FileUploadRepository + Clone,
    C: Clock + Clone
{
    pub fn new(repository: FR, limiter: FileSizeLimiter, settings: &Settings, clock: C) -> Self {
        Self { repository, limiter, settings: settings.clone(), clock }
    }

    pub fn register_file(&self, file_create: FileCreate) -> Result<FileUpload, FileUploadError> {
        if self.limiter.check_file_size(file_create.size as u64) == false {
            return Err(FileUploadError::MaxFileSizeExceeded)
        }

        let now = self.clock.now();
        let expires_at = match self.settings.upload.expires_in_seconds {
            Some(expires_in) => now.checked_add_signed(Duration::seconds(expires_in as i64)),
            None => None
        };

        let file_upload = FileUpload {
            id: file_create.id,
            name: file_create.name,
            mime_type: file_create.mime_type,
            size: file_create.size,
            uploaded_at: now,
            chunked: false,
            expires_at
        };

        let existing = self.repository.find_by_id(&file_create.id)?;

        if existing.is_some() {
            return Err(FileUploadError::Exists)
        }

        self.repository.save(&file_upload.clone())?;

        Ok(file_upload)
    }
}

impl FileRegister<PostgresFileUploadRepository<MokaCache>, SystemClock> {
    pub fn get_with_settings(settings: &Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(&settings),
            FileSizeLimiter::new(settings),
            settings,
            SystemClock,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::model::FileUpload;
    use crate::repository::InMemoryFileUploadRepository;
    use crate::routes::FileCreate;
    use crate::services::{FakeClock, FileRegister, FileSizeLimiter, FileUploadError};
    use std::collections::HashMap;
    use chrono::{Duration, NaiveDateTime};
    use uuid::Uuid;
    use crate::config::Settings;

    fn test_date_time() -> NaiveDateTime {
        NaiveDateTime::from_timestamp(1676224000, 0)
    }

    impl FileRegister<InMemoryFileUploadRepository, FakeClock> {
        pub fn create() -> Self {
            Self::new(
                Self::create_repository(),
                FileSizeLimiter::create(),
                &Settings::default(),
                FakeClock(test_date_time())
            )
        }

        pub fn create_limited() -> Self {
            Self::new(
                Self::create_repository(),
                FileSizeLimiter::create_limited(),
                &Settings::default(),
                FakeClock(test_date_time())
            )
        }

        pub fn create_with_expiration() -> Self {
            let mut settings = Settings::default();
            settings.upload.expires_in_seconds = Some(86400);

            Self::new(
                Self::create_repository(),
                FileSizeLimiter::create_limited(),
                &settings,
                FakeClock(test_date_time())
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

        let file_create_clone = file_create.clone();
        let file_upload = FileUpload {
            id,
            name: file_create_clone.name,
            mime_type: file_create_clone.mime_type,
            size: file_create_clone.size,
            uploaded_at: test_date_time(),
            chunked: false,
            expires_at: None
        };

        let result = register.register_file(file_create);
        assert!(result.is_ok());
        assert_eq!(file_upload, result.unwrap());
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

    #[test]
    fn test_register_file_with_expires_in_seconds() {
        let register = FileRegister::create_with_expiration();
        let id = Uuid::new_v4();
        let file_create = FileCreate {
            id,
            name: String::from("test.txt"),
            mime_type: String::from("text/plain"),
            size: 100
        };

        let file_create_clone = file_create.clone();
        let file_upload = FileUpload {
            id,
            name: file_create_clone.name,
            mime_type: file_create_clone.mime_type,
            size: file_create_clone.size,
            uploaded_at: test_date_time(),
            chunked: false,
            expires_at: test_date_time().checked_add_signed(Duration::seconds(86400))
        };

        let result = register.register_file(file_create);
        assert!(result.is_ok());
        assert_eq!(file_upload, result.unwrap());
    }
}
