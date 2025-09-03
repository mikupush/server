#[cfg(test)]
pub mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use chrono::Utc;
    use diesel::RunQueryDsl;
    use uuid::Uuid;
    use crate::config::Upload;
    use crate::database::DbPool;
    use crate::model::FileUpload;
    use crate::schema::file_uploads;

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

        let settings = Upload::test_default();
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
}
