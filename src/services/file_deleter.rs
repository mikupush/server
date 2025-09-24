/// Copyright 2025 Miku Push! Team
///
/// Licensed under the Apache License, Version 2.0 (the "License");
/// you may not use this file except in compliance with the License.
/// You may obtain a copy of the License at
///
///     http://www.apache.org/licenses/LICENSE-2.0
///
/// Unless required by applicable law or agreed to in writing, software
/// distributed under the License is distributed on an "AS IS" BASIS,
/// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
/// See the License for the specific language governing permissions and
/// limitations under the License.

use std::path::Path;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use tracing::debug;
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
        debug!("deleting file with id: {}", id.to_string());
        let mut connection = self.pool.get()?;
        let file_upload: Option<FileUpload> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()?;

        let Some(file_upload) = file_upload else {
            debug!("file with id {} does not exist on the database", id.to_string());
            return Err(FileDeleteError::NotExists { id });
        };

        let directory = self.settings.upload.directory().clone();
        let path = Path::new(&directory).join(file_upload.name);

        if path.exists() {
            debug!("deleting file from the filesystem: {} ({})", path.display(), id.to_string());
            std::fs::remove_file(path)?;
        }

        debug!("deleting file from the database: {}", id.to_string());
        diesel::delete(file_uploads::table.find(id))
            .execute(&mut connection)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::database::DbPool;
    use crate::services::FileDeleter;

    impl FileDeleter {
        pub fn test(pool: DbPool) -> Self {
            Self::new(pool, Settings::default())
        }
    }
}
