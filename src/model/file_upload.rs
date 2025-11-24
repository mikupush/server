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

use crate::domain::FileUpload as DomainFileUpload;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = crate::schema::file_uploads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileUpload {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub chunked: bool
}

impl From<FileUpload> for DomainFileUpload {
    fn from(model: FileUpload) -> Self {
        Self {
            id: model.id,
            name: model.name,
            mime_type: model.mime_type,
            size: model.size,
            uploaded_at: model.uploaded_at,
            chunked: model.chunked,
        }
    }
}

impl From<DomainFileUpload> for FileUpload {
    fn from(domain: DomainFileUpload) -> Self {
        Self {
            id: domain.id,
            name: domain.name,
            mime_type: domain.mime_type,
            size: domain.size,
            uploaded_at: domain.uploaded_at,
            chunked: domain.chunked,
        }
    }
}
