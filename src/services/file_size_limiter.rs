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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::config::Upload;

    impl FileSizeLimiter {
        pub fn test_limited(max_size: u64) -> Self {
            let mut settings = Settings::default();
            settings.upload = Upload::with_size(max_size);
            Self::new(settings)
        }

        pub fn test_unlimited() -> Self {
            Self::new(Settings::default())
        }
    }
}
