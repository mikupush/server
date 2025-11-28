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
use crate::errors::FileDeleteError;
use crate::repository::{FileUploadRepository, PostgresFileUploadRepository};
use crate::services::object_storage_remover::{FileSystemObjectStorageRemover, ObjectStorageRemoveError, ObjectStorageRemover};
use std::path::Path;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileDeleter<FR, OSR>
where
    FR: FileUploadRepository + Clone,
    OSR: ObjectStorageRemover + Clone,
{
    repository: FR,
    remover: OSR,
    settings: Settings,
}

impl<FR, OSR> FileDeleter<FR, OSR>
where
    FR: FileUploadRepository + Clone,
    OSR: ObjectStorageRemover + Clone,
{
    pub fn new(repository: FR, object_storage_remover: OSR, settings: Settings) -> Self {
        Self { repository, remover: object_storage_remover, settings }
    }

    pub fn delete(&self, id: Uuid) -> Result<(), FileDeleteError> {
        debug!("deleting file with id: {}", id.to_string());
        let file_upload = match self.repository.find_by_id(id)? {
            Some(file_upload) => file_upload,
            None => {
                debug!("file with id {} does not exist on the database", id.to_string());
                return Err(FileDeleteError::NotExists { id });
            }
        };

        let directory = file_upload.directory(&self.settings)?;

        debug!("deleting file from the filesystem: {} ({})", directory.display(), id.to_string());
        let remove_result = self.remover.remove(directory.to_string_lossy().to_string());
        if let Err(err) = remove_result && err != ObjectStorageRemoveError::NotExists {
            return Err(err.into());
        }

        debug!("deleting file from the database: {}", id.to_string());
        self.repository.delete(id)?;

        Ok(())
    }
}

impl FileDeleter<PostgresFileUploadRepository, FileSystemObjectStorageRemover> {
    pub fn get_with_settings(settings: Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(settings.clone()),
            FileSystemObjectStorageRemover::new(),
            settings
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::errors::FileDeleteError;
    use crate::model::FileUpload;
    use crate::repository::{FileUploadRepository, InMemoryFileUploadRepository};
    use crate::services::object_storage_remover::{FakeObjectStorageRemover, ObjectStorageRemoveError, ObjectStorageRemover};
    use crate::services::FileDeleter;
    use serial_test::serial;
    use std::collections::HashMap;
    use std::path::Path;
    use uuid::Uuid;

    impl FileDeleter<InMemoryFileUploadRepository, FakeObjectStorageRemover> {
        pub fn create() -> Self {
            Self::new(
                Self::create_repository(),
                FakeObjectStorageRemover::new(),
                Settings::default()
            )
        }

        fn create_repository() -> InMemoryFileUploadRepository {
            let items = HashMap::from([
                (
                    Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap(),
                    FileUpload::create("5769aa43-2380-49be-aafb-e9dd4bd4564f")
                ),
            ]);

            InMemoryFileUploadRepository::new(items)
        }
    }

    #[test]
    #[serial]
    fn test_delete_existing_file() {
        let deleter = FileDeleter::create();
        let id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();

        let result = deleter.delete(id);

        assert!(result.is_ok());
        let stored = deleter.repository.find_by_id(id).unwrap();
        assert!(stored.is_none(), "file upload should be removed from repository");
        cleanup_directory(&deleter.settings.upload.directory(), id);
    }

    #[test]
    #[serial]
    fn test_delete_not_exists() {
        let deleter = FileDeleter::create();
        let missing_id = Uuid::parse_str("f5dcca7d-e8ba-4e87-b7b1-e3cde6bc857d").unwrap();

        let result = deleter.delete(missing_id);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FileDeleteError::NotExists { id } if id == missing_id));

        let existing_id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();
        assert!(deleter.repository.find_by_id(existing_id).unwrap().is_some(), "existing file should stay untouched");
    }

    #[test]
    #[serial]
    fn test_delete_ignores_missing_storage_object() {
        let repository = FileDeleter::<InMemoryFileUploadRepository, FakeObjectStorageRemover>::create_repository();
        let settings = Settings::default();
        let deleter = FileDeleter::new(repository.clone(), NotFoundObjectStorageRemover, settings.clone());
        let id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();

        let result = deleter.delete(id);

        assert!(result.is_ok(), "removal should continue when storage object is missing");
        assert!(repository.find_by_id(id).unwrap().is_none(), "record should still be deleted from repository");
        cleanup_directory(&settings.upload.directory(), id);
    }

    #[test]
    #[serial]
    fn test_delete_returns_error_on_storage_failure() {
        let repository = FileDeleter::<InMemoryFileUploadRepository, FakeObjectStorageRemover>::create_repository();
        let settings = Settings::default();
        let deleter = FileDeleter::new(repository.clone(), FailingObjectStorageRemover, settings.clone());
        let id = Uuid::parse_str("5769aa43-2380-49be-aafb-e9dd4bd4564f").unwrap();

        let result = deleter.delete(id);

        assert!(matches!(result, Err(FileDeleteError::IO { .. })));
        assert!(repository.find_by_id(id).unwrap().is_some(), "record should remain when storage removal fails");
        cleanup_directory(&settings.upload.directory(), id);
    }

    #[derive(Clone)]
    struct NotFoundObjectStorageRemover;

    impl ObjectStorageRemover for NotFoundObjectStorageRemover {
        fn remove(&self, _location: String) -> Result<(), ObjectStorageRemoveError> {
            Err(ObjectStorageRemoveError::NotExists)
        }
    }

    #[derive(Clone)]
    struct FailingObjectStorageRemover;

    impl ObjectStorageRemover for FailingObjectStorageRemover {
        fn remove(&self, _location: String) -> Result<(), ObjectStorageRemoveError> {
            Err(ObjectStorageRemoveError::IO("unexpected error".to_string()))
        }
    }

    fn cleanup_directory(base: &str, id: Uuid) {
        let directory = Path::new(base).join(id.to_string());
        if directory.exists() {
            let _ = std::fs::remove_dir_all(directory);
        }
    }
}
