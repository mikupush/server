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
use tracing::warn;

pub fn read_template(settings: &Settings, template: &str) -> String {
    let template_dir = settings.server.templates_directory();
    let path = std::path::Path::new(&template_dir).join(template);

    if !path.exists() {
        warn!("template file {} does not exist", path.display());
        return "".to_string();
    }

    match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(err) => {
            warn!("failed to read template file {}: {}", path.display(), err);
            "".to_string()
        }
    }
}

#[cfg(test)]
pub mod tests {
    use crate::config::Upload;
    use crate::database::DbPool;
    use crate::domain::FileUpload;
    use crate::model::FileUpload as FileUploadModel;
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
            uploaded_at: Utc::now().naive_utc(),
            chunked: false
        };

        let settings = Upload::default();
        let path = Path::new(&settings.directory())
            .join(file_upload.id.to_string())
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
            chunked: false
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
