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

mod file_upload_error;
mod file_delete_error;
mod file_read_error;
mod route_error;
mod file_info_error;

pub use file_delete_error::*;
pub use file_read_error::*;
pub use file_upload_error::*;
pub use route_error::*;
pub use file_info_error::*;

use std::fmt::Display;

pub trait Error: Display {
    fn code(&self) -> String;
    fn message(&self) -> String;
}