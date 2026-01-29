pub struct FilePart;

impl FilePart {
    pub fn name(index: usize) -> String {
        format!("{}.part", index)
    }
}
