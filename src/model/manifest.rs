use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Manifest {
    pub upload_id: Uuid,
    pub parts: Vec<Part>
}

#[derive(Debug, Clone)]
pub struct Part {
    pub id: Uuid,
    pub index: i64,
    pub upload_id: Uuid,
}

impl Part {
    pub fn new(upload_id: Uuid, index: i64) -> Self {
        Self { index, upload_id, id: Uuid::new_v4() }
    }

    pub fn file_name(&self) -> String {
        format!("{}.part", self.id)
    }
}
