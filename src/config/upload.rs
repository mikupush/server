use log::debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upload {
    max_size: Option<u64>
}

impl Upload {
    /// Returns true if the upload is limited.
    /// If true, you can unwrap securely the max_size optional
    pub fn is_limited(&self) -> bool {
        self.max_size.is_some()
    }

    pub fn max_size(&self) -> Option<u64> {
        let value = std::env::var("MIKU_PUSH_UPLOAD_MAX_SIZE").ok();
        if let Some(value) = value {
            debug!("using env variable MIKU_PUSH_UPLOAD_MAX_SIZE: {}", value);
            return Some(value.parse::<u64>().expect("upload max size must be a number"))
        }

        let value = self.max_size.clone();
        if let Some(value) = value {
            debug!("using upload.max_size configuration: {}", value);
            return Some(value)
        }

        None
    }
}

impl Default for Upload {
    fn default() -> Self {
        Self {
            max_size: None
        }
    }
}
