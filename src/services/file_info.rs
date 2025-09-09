use std::path::Path;
use diesel::{OptionalExtension, QueryDsl, RunQueryDsl};
use log::debug;
use uuid::Uuid;
use crate::config::Settings;
use crate::database::DbPool;
use crate::errors::FileInfoError;
use crate::model::{FileInfo, FileUpload};
use crate::schema::file_uploads;

#[derive(Debug, Clone)]
pub struct FileInfoFinder {
    pool: DbPool,
    settings: Settings,
}

impl FileInfoFinder {
    pub fn new(pool: DbPool, settings: Settings) -> Self {
        Self { pool, settings }
    }

    pub fn find(&self, id: Uuid) -> Result<FileInfo, FileInfoError> {
        debug!("retrieving file info for file with id: {}", id.to_string());
        let mut connection = self.pool.get()?;
        let file_upload: Option<FileUpload> = file_uploads::table
            .find(id)
            .first(&mut connection)
            .optional()?;

        let Some(file_upload) = file_upload else {
            debug!("file with id {} does not exist on the database", id.to_string());
            return Err(FileInfoError::NotExists { id });
        };

        let directory = self.settings.upload.directory().clone();
        let path = Path::new(&directory).join(file_upload.clone().name);

        Ok(FileInfo::from_file_upload(&file_upload, path))
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Settings;
    use crate::database::DbPool;
    use super::*;

    impl FileInfoFinder {
        pub fn test(pool: DbPool) -> Self {
            Self::new(pool, Settings::default())
        }
    }
}
