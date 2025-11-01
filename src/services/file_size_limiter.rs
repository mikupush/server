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

use tracing::debug;
use crate::config::Settings;
use crate::errors::FileUploadError;

#[derive(Debug, Clone)]
pub struct FileSizeLimiter {
    settings: Settings
}

impl FileSizeLimiter {
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn check_file_size(&self, file_size: u64) -> Result<(), FileUploadError> {
        if !self.settings.upload.is_limited() {
            return Ok(());
        }

        let limit = self.settings.upload.max_size().unwrap();
        debug!("file size is limited by: {} bytes", limit);

        if file_size > limit {
            debug!("file size limit exceeded: {} > {} bytes", file_size, limit);
            return Err(FileUploadError::MaxFileSizeExceeded)
        }

        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    impl FileSizeLimiter {
        pub fn test() -> Self {
            Self::new(Settings::default())
        }
    }
}