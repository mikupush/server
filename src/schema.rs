// @generated automatically by Diesel CLI.

diesel::table! {
    file_uploads (id) {
        id -> Uuid,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 128]
        mime_type -> Varchar,
        size -> Int8,
        uploaded_at -> Timestamp,
    }
}
