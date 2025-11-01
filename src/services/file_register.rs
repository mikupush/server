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

#[cfg(test)]
mod tests {
    use crate::database::tests::create_test_database_connection;
    use crate::services::{FileRegister, FileSizeLimiter};

    impl FileRegister {
        pub fn test() -> Self {
            Self::new(
                create_test_database_connection(),
                FileSizeLimiter::test()
            )
        }
    }
}