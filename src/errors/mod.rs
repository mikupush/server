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
