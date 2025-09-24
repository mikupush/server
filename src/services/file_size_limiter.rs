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
