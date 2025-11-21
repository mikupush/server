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

    /// Check if the file size is limited by the settings
    /// return true if the file size is not limited or the file size is not exceeded
    /// false if the file size is exceeded
    pub fn check_file_size(&self, file_size: u64) -> bool {
        if !self.settings.upload.is_limited() {
            return true;
        }

        let limit = self.settings.upload.max_size().unwrap();
        debug!("file size is limited by: {} bytes", limit);

        if file_size > limit {
            debug!("file size limit exceeded: {} > {} bytes", file_size, limit);
            return false
        }

        true
    }
}

#[cfg(test)]
pub mod tests {
    use crate::config::{DataBase, LoggingConfig, Server, Upload};
    use super::*;

    impl FileSizeLimiter {
        pub fn create() -> Self {
            Self::new(Settings::default())
        }

        pub fn create_limited() -> FileSizeLimiter {
            let settings = Settings::new(
                Server::default(),
                LoggingConfig::default(),
                DataBase::default(),
                Upload::new(Some(100), None)
            );

            FileSizeLimiter::new(settings)
        }

        pub fn create_unlimited() -> FileSizeLimiter {
            let settings = Settings::new(
                Server::default(),
                LoggingConfig::default(),
                DataBase::default(),
                Upload::new(None, None)
            );

            FileSizeLimiter::new(settings)
        }
    }

    #[test]
    fn test_check_file_size() {
        let limiter = FileSizeLimiter::create_limited();

        assert_eq!(true, limiter.check_file_size(100));
        assert_eq!(false, limiter.check_file_size(1000));
    }

    #[test]
    fn test_check_file_unlimited() {
        let limiter = FileSizeLimiter::create_unlimited();

        assert_eq!(true, limiter.check_file_size(100));
        assert_eq!(true, limiter.check_file_size(1000));
    }
}
