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

use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use uuid::Uuid;
use crate::database::DbPool;
use crate::domain::FileUpload;
use crate::model::FileUpload as FileUploadModel;
use crate::schema::file_uploads;
use diesel::result::Error as DieselError;
use r2d2::Error as PoolError;

#[derive(Debug)]
pub enum FileUploadRepositoryError {
    Pool(PoolError),
    Db(DieselError),
}

impl From<PoolError> for FileUploadRepositoryError {
    fn from(value: PoolError) -> Self {
        Self::Pool(value)
    }
}

impl From<DieselError> for FileUploadRepositoryError {
    fn from(value: DieselError) -> Self {
        Self::Db(value)
    }
}

pub trait FileUploadRepository {
    fn find_by_id(&self, file_upload_id: Uuid) -> Result<Option<FileUpload>, FileUploadRepositoryError>;
    fn delete(&self, file_upload_id: Uuid) -> Result<(), FileUploadRepositoryError>;
    fn save(&self, file_upload: FileUpload) -> Result<(), FileUploadRepositoryError>;
}

#[derive(Debug, Clone)]
pub struct PostgresFileUploadRepository {
    db_pool: DbPool
}

impl PostgresFileUploadRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }
}

impl FileUploadRepository for PostgresFileUploadRepository {
    fn find_by_id(&self, file_upload_id: Uuid) -> Result<Option<FileUpload>, FileUploadRepositoryError> {
        let mut connection = self.db_pool.get()?;
        let record: Option<FileUploadModel> = file_uploads::table
            .find(file_upload_id)
            .first(&mut connection)
            .optional()?;

        Ok(record.map(FileUpload::from))
    }

    fn delete(&self, file_upload_id: Uuid) -> Result<(), FileUploadRepositoryError> {
        let mut connection = self.db_pool.get()?;
        diesel::delete(file_uploads::table.find(file_upload_id))
            .execute(&mut connection)?;

        Ok(())
    }

    fn save(&self, file_upload: FileUpload) -> Result<(), FileUploadRepositoryError> {
        let mut connection = self.db_pool.get()?;
        let model: FileUploadModel = file_upload.into();
        diesel::insert_into(file_uploads::table)
            .values(&model)
            .execute(&mut connection)?;

        Ok(())
    }
}
