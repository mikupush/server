use std::path::PathBuf;
use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = crate::schema::file_uploads)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct FileUpload {
    pub id: Uuid,
    pub name: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: NaiveDateTime
}

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
    pub status: FileStatus
}

impl FileInfo {
    pub fn from_file_upload(file_upload: &FileUpload, path: PathBuf) -> Self {
        Self {
            id: file_upload.id,
            name: file_upload.name.clone(),
            mime_type: file_upload.mime_type.clone(),
            size: file_upload.size,
            uploaded_at: file_upload.uploaded_at,
            status: match path.exists() {
                true => FileStatus::Uploaded,
                false => FileStatus::WaitingForUpload,
            }
        }
    }
}
