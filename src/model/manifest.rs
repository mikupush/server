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

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Manifest {
    pub upload_id: Uuid,
    pub parts: Vec<Part>
}

#[derive(Debug, Clone)]
pub struct Part {
    pub id: Uuid,
    pub index: i64,
    pub upload_id: Uuid,
    pub size: u64,
}

impl Part {
    pub fn new(upload_id: Uuid, index: i64) -> Self {
        Self { index, upload_id, id: Uuid::new_v4(), size: 0 }
    }

    pub fn file_name(&self) -> String {
        format!("{}.part", self.id)
    }
}