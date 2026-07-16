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

use crate::config::Settings;
use crate::file::error::FileInfoError;
use crate::file::{FileUploadRepository, PostgresFileUploadRepository};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tracing::debug;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::NaiveDateTime;
use crate::cache::{Cache, MokaCache};
use crate::file::upload::FileUpload;
use crate::schema::file_uploads::chunked;
use crate::storage::{FileSystemObjectStorageCounter, ObjectStorageCounter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileStatus {
    WaitingForUpload,
    Uploaded
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime,
    pub status: FileStatus,
    pub expires_at: Option<NaiveDateTime>,
    pub chunked: bool,
    pub chunks: Option<usize>,
}

impl FileInfo {
    pub fn from_file_upload(file_upload: &FileUpload, path: &PathBuf) -> Self {
        Self {
            id: file_upload.id,
            name: file_upload.name.clone(),
            mime_type: file_upload.mime_type.clone(),
            size: file_upload.size,
            uploaded_at: file_upload.uploaded_at,
            status: match path.exists() {
                true => FileStatus::Uploaded,
                false => FileStatus::WaitingForUpload,
            },
            expires_at: file_upload.expires_at,
            chunked: file_upload.chunked,
            chunks: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileInfoFinder<FR, OC, C>
where
    FR: FileUploadRepository + Clone,
    OC: ObjectStorageCounter + Clone,
    C: Cache + Clone,
{
    repository: FR,
    settings: Settings,
    object_counter: OC,
    cache: C,
}

impl<FR, OC, C> FileInfoFinder<FR, OC, C>
where
    FR: FileUploadRepository + Clone,
    OC: ObjectStorageCounter + Clone,
    C: Cache + Clone
{
    pub fn new(repository: FR, object_counter: OC, cache: C, settings: &Settings) -> Self {
        Self { repository, settings: settings.clone(), object_counter, cache }
    }

    pub fn find(&self, id: &Uuid) -> Result<FileInfo, FileInfoError> {
        debug!("retrieving file info for file with id: {}", id.to_string());
        let file_upload = match self.repository.find_by_id(id)? {
            Some(file_upload) => file_upload,
            None => {
                debug!("file with id {} does not exist on the database", id.to_string());
                return Err(FileInfoError::NotExists { id: id.clone() });
            }
        };

        let directory = file_upload.content_directory(&self.settings)?;

        let info = if file_upload.chunked {
            self.build_file_info_from_chunked_file(&file_upload, &directory)?
        } else {
            self.build_file_info_from_single_file(&file_upload, &directory)
        };

        Ok(info)
    }

    fn build_file_info_from_chunked_file(
        &self,
        file_upload: &FileUpload,
        directory: &PathBuf
    ) -> std::io::Result<FileInfo> {
        let cache_chunk_count_key = format!("mikupush:upload:{}:chunk_count", file_upload.id);
        let mut info = FileInfo::from_file_upload(&file_upload, &directory);
        let directory_str = &directory.to_string_lossy().to_string();
        let cached_chunk_count: Option<usize> = self.cache.get(&cache_chunk_count_key);

        if let Some(chunk_count) = cached_chunk_count {
            info.chunks = Some(chunk_count);
            return Ok(info);
        }

        let chunk_count = self.object_counter.count_in_directory(directory_str)?;
        self.cache.set(&cache_chunk_count_key, chunk_count, Some(Duration::from_mins(1)));
        info.chunks = Some(chunk_count);
        Ok(info)
    }

    fn build_file_info_from_single_file(
        &self,
        file_upload: &FileUpload,
        directory: &PathBuf
    ) -> FileInfo {
        let path = Path::new(&directory).join(file_upload.clone().name);
        FileInfo::from_file_upload(&file_upload, &path)
    }
}

impl FileInfoFinder<
    PostgresFileUploadRepository<MokaCache>,
    FileSystemObjectStorageCounter,
    MokaCache
> {
    pub fn get_with_settings(settings: &Settings) -> Self {
        Self::new(
            PostgresFileUploadRepository::get_with_settings(settings),
            FileSystemObjectStorageCounter::new(),
            MokaCache::current(),
            settings
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::NoOpCache;
    use super::*;
    use crate::config::Settings;
    use crate::database::DbPool;
    use crate::file::PostgresFileUploadRepository;

    impl FileInfoFinder<
        PostgresFileUploadRepository<NoOpCache>,
        FileSystemObjectStorageCounter,
        NoOpCache
    > {
        pub fn test(pool: DbPool) -> Self {
            Self::new(
                PostgresFileUploadRepository::with_pool(pool),
                FileSystemObjectStorageCounter::new(),
                NoOpCache,
                &Settings::default()
            )
        }
    }
}
