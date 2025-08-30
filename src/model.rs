use chrono::NaiveDateTime;
use diesel::{Insertable, Queryable};
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
