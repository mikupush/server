use std::fs;

pub trait ObjectStorageCounter {
    fn count_in_directory(&self, directory: &String) -> Result<usize, std::io::Error>;
}

#[derive(Debug, Clone)]
pub struct FileSystemObjectStorageCounter;

impl FileSystemObjectStorageCounter {
    pub fn new() -> Self {
        Self
    }
}

impl ObjectStorageCounter for FileSystemObjectStorageCounter {
    fn count_in_directory(&self, directory: &String) -> Result<usize, std::io::Error> {
        let directory_info = fs::read_dir(directory)?;
        Ok(directory_info.count())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use std::fs;
    use super::{FileSystemObjectStorageCounter, ObjectStorageCounter};

    #[test]
    fn test_count_in_directory() {
        let directory = setup_test_directory();
        let counter = FileSystemObjectStorageCounter::new();

        let expected = 5;
        let result = counter.count_in_directory(&directory).unwrap();

        assert_eq!(expected, result, "expected object count {} and got {}", expected, result);
    }

    fn setup_test_directory() -> String {
        let id = Uuid::new_v4();
        let directory = format!("tmp/FileSystemObjectStorageCounterTest/{}", id);
        fs::create_dir_all(&directory).unwrap();

        for i in 0..5 {
            fs::File::create(format!("{}/{}", directory, i)).unwrap();
        }

        directory
    }
}
