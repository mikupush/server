use log::debug;
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
