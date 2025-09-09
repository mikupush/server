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
