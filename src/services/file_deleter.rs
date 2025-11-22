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
use crate::repository::FileUploadRepository;
use std::path::Path;
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct FileDeleter<FR>
where
    FR: FileUploadRepository + Clone,
{
    repository: FR,
    settings: Settings,
}

impl<FR> FileDeleter<FR>
where
    FR: FileUploadRepository + Clone,
{
    pub fn new(repository: FR, settings: Settings) -> Self {
        Self { repository, settings }
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
        let path = Path::new(&directory).join(file_upload.name);

        if path.exists() {
            debug!("deleting file from the filesystem: {} ({})", path.display(), id.to_string());
            std::fs::remove_file(path)?;
        }

        debug!("deleting file from the database: {}", id.to_string());
        self.repository.delete(id)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::database::DbPool;
    use crate::repository::PostgresFileUploadRepository;
    use crate::services::FileDeleter;

    impl FileDeleter<PostgresFileUploadRepository> {
        pub fn test(pool: DbPool) -> Self {
            Self::new(PostgresFileUploadRepository::new(pool), Settings::default())
        }
    }
}
