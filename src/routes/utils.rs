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

#[cfg(test)]
pub mod tests {
    use crate::config::Upload;
    use crate::database::DbPool;
    use crate::model::FileUpload;
    use crate::schema::file_uploads;
    use actix_web::dev::ServiceResponse;
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use uuid::Uuid;

    // used to give unique prefix to the test file
    static TEST_FILE_COUNT: Mutex<i32> = Mutex::new(0);

    pub fn create_test_file_upload(pool: DbPool) -> (PathBuf, FileUpload) {
        let mut count = TEST_FILE_COUNT.lock().unwrap();
        let file_upload = FileUpload {
            id: Uuid::new_v4(),
            name: format!("hatsune_miku_{}.jpg", count),
            mime_type: "image/jpeg".to_string(),
            size: 200792,
            uploaded_at: Utc::now().naive_utc()
        };

        let settings = Upload::default();
        let path = Path::new(&settings.directory())
            .join(file_upload.name.clone());
        std::fs::write(path.clone(), vec![1; file_upload.size as usize]).unwrap();

        let mut connection = pool.get().unwrap();
        diesel::insert_into(file_uploads::table)
            .values(&file_upload)
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
            uploaded_at: Utc::now().naive_utc()
        };

        let mut connection = pool.get().unwrap();
        diesel::insert_into(file_uploads::table)
            .values(&file_upload)
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
