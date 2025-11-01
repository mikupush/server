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