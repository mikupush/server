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

use actix_web::HttpRequest;

pub fn range_header(request: &HttpRequest, total_size: u64) -> Option<(u64, u64)> {
    let range = request.headers().get("Range")
        .and_then(|value| value.to_str().ok())?;

    if !range.starts_with("bytes=") {
        return None;
    }

    let range = &range["bytes=".len()..];
    let parts: Vec<&str> = range.split('-').collect();

    if parts.len() != 2 {
        return None;
    }

    let (start, end) = match (parts[0], parts[1]) {
        ("", suffix) => {
            let suffix = suffix.parse::<u64>().ok()?;
            let start = total_size.saturating_sub(suffix);
            (start, total_size)
        }
        (start, "") => {
            let start = start.parse::<u64>().ok()?;
            (start, total_size)
        }
        (start, end) => {
            let start = start.parse::<u64>().ok()?;
            let end = end.parse::<u64>().ok()?;
            (start, end)
        }
    };

    if start >= total_size || start > end {
        return None;
    }

    let end = std::cmp::min(end, total_size.saturating_sub(1));

    Some((start, end))
}

#[cfg(test)]
pub mod tests {
    use std::io::Cursor;
    use crate::config::Settings;
    use crate::database::DbPool;
    use crate::file::{FileRegister, FileUpload, FileUploadModel, FileUploader};
    use crate::schema::file_uploads;
    use actix_web::dev::ServiceResponse;
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use uuid::Uuid;
    use crate::routes::FileCreate;

    // used to give unique prefix to the test file
    static TEST_FILE_COUNT: Mutex<i32> = Mutex::new(0);
    pub const TEST_FILE_CONTENT_LENGTH: usize = 1024;

    pub struct IntegrationTestFileUploadFactory {
        settings: Settings,
        pool: DbPool,
    }

    pub type FileCreateStub = (Vec<u8>, FileCreate);

    impl IntegrationTestFileUploadFactory {
        pub fn new(settings: &Settings, pool: &DbPool) -> Self {
            Self { settings: settings.clone(), pool: pool.clone() }
        }

        pub async fn create(&self, stub: FileCreateStub) -> (PathBuf, FileUpload) {
            let register = FileRegister::for_integration(&self.settings, &self.pool);
            let uploader = FileUploader::for_integration(&self.settings, &self.pool);
            let (content, request) = stub;
            let reader = Cursor::new(content);

            let upload = register.register_file(request.clone()).unwrap();
            let _ = uploader.upload_file(request.id, reader).await;

            (upload.content_path(&self.settings).unwrap(), upload)
        }

        pub async fn create_chunked(&self, stub: FileCreateStub) -> FileUpload {
            let register = FileRegister::for_integration(&self.settings, &self.pool);
            let uploader = FileUploader::for_integration(&self.settings, &self.pool);
            let (content, request) = stub;
            let chunks: Vec<&[u8]> = content.chunks(2).collect();

            let upload = register.register_file(request.clone()).unwrap();

            for (index, chunk) in chunks.iter().enumerate() {
                let reader = Cursor::new(chunk);
                let _ = uploader.upload_chunk(request.id, index as i64, reader).await;
            }

            upload
        }
    }

    pub struct FileCreateFactories;

    impl FileCreateFactories {
        pub fn text_plain() -> (Vec<u8>, FileCreate) {
            let content = vec![0u8; TEST_FILE_CONTENT_LENGTH];
            let request = FileCreate {
                id: Uuid::new_v4(),
                name: "example_file.txt".to_string(),
                mime_type: "text/plain".to_string(),
                size: content.len() as i64,
            };

            (content, request)
        }

        pub fn image_png() -> (Vec<u8>, FileCreate) {
            let content = vec![0u8; TEST_FILE_CONTENT_LENGTH];
            let request = FileCreate {
                id: Uuid::new_v4(),
                name: "example_image.png".to_string(),
                mime_type: "image/png".to_string(),
                size: content.len() as i64,
            };

            (content, request)
        }

        pub fn video_mp4() -> (Vec<u8>, FileCreate) {
            let content = vec![0u8; TEST_FILE_CONTENT_LENGTH];
            let request = FileCreate {
                id: Uuid::new_v4(),
                name: "example_video.mp4".to_string(),
                mime_type: "video/mp4".to_string(),
                size: content.len() as i64,
            };

            (content, request)
        }
    }

    pub fn create_test_file_upload(pool: DbPool) -> (PathBuf, FileUpload) {
        let mut count = TEST_FILE_COUNT.lock().unwrap();
        let file_upload = FileUpload {
            id: Uuid::new_v4(),
            name: format!("hatsune_miku_{}.jpg", count),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
            uploaded_at: Utc::now().naive_utc(),
            chunked: false,
            expires_at: None
        };

        let settings = Settings::default();
        let path = file_upload.content_directory(&settings)
            .unwrap()
            .join(file_upload.name.clone());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path.clone(), vec![1; file_upload.size as usize]).unwrap();

        let mut connection = pool.get().unwrap();
        let record: FileUploadModel = file_upload.clone().into();
        diesel::insert_into(file_uploads::table)
            .values(&record)
            .execute(&mut connection)
            .unwrap();

        *count += 1;
        (path, file_upload)
    }

    pub fn register_test_file(pool: DbPool) -> FileUpload {
        let file_upload = FileUpload {
            id: Uuid::new_v4(),
            name: format!("hatsune_miku_{}.jpg", Utc::now().timestamp()),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
            uploaded_at: Utc::now().naive_utc(),
            chunked: false,
            expires_at: None
        };

        let mut connection = pool.get().unwrap();
        let record: FileUploadModel = file_upload.clone().into();
        diesel::insert_into(file_uploads::table)
            .values(&record)
            .execute(&mut connection)
            .unwrap();

        file_upload
    }

    pub fn header_value(header: &str, response: &ServiceResponse) -> String {
        let Some(value) = response.headers().get(header) else {
            return "".to_string()
        };

        match value.to_str() {
            Ok(value) => value.to_string(),
            Err(_) => "".to_string()
        }
    }
}
