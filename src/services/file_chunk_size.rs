use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use uuid::Uuid;

pub trait ChunkedUploadSizeAccumulator: Clone {
    fn accumulate(&self, id: Uuid, size: u64) -> u64;
    fn get_total(&self, id: Uuid) -> Option<u64>;
    fn remove(&self, id: Uuid);
}

static CHUNKED_UPLOAD_SIZE_STATE: LazyLock<Mutex<HashMap<Uuid, u64>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Clone)]
pub struct InMemoryChunkedUploadSizeAccumulator;

impl InMemoryChunkedUploadSizeAccumulator {
    pub fn new() -> Self {
        Self {}
    }
}

impl ChunkedUploadSizeAccumulator for InMemoryChunkedUploadSizeAccumulator {
    fn accumulate(&self, id: Uuid, size: u64) -> u64 {
        let mut state = CHUNKED_UPLOAD_SIZE_STATE.lock().unwrap();
        let size = state.entry(id).and_modify(|s| *s += size).or_insert(size);
        *size
    }

    fn get_total(&self, id: Uuid) -> Option<u64> {
        let state = CHUNKED_UPLOAD_SIZE_STATE.lock().unwrap();
        state.get(&id).copied()
    }

    fn remove(&self, id: Uuid) {
        let mut state = CHUNKED_UPLOAD_SIZE_STATE.lock().unwrap();
        state.remove(&id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_accumulate() {
        let accumulator = InMemoryChunkedUploadSizeAccumulator::new();
        let id = Uuid::new_v4();

        let size = accumulator.accumulate(id, 100);
        assert_eq!(100, size);

        let size = accumulator.accumulate(id, 100);
        assert_eq!(200, size);
    }

    #[test]
    fn test_in_memory_get_total() {
        let accumulator = InMemoryChunkedUploadSizeAccumulator::new();
        let id = Uuid::new_v4();

        let size = accumulator.accumulate(id, 100);
        assert_eq!(100, size);

        let size = accumulator.get_total(id);
        assert_eq!(100, size.unwrap());
    }

    #[test]
    fn test_in_memory_get_total_not_exist() {
        let accumulator = InMemoryChunkedUploadSizeAccumulator::new();
        let id = Uuid::new_v4();

        let size = accumulator.get_total(id);
        assert_eq!(None, size);
    }

    #[test]
    fn test_in_memory_remove() {
        let accumulator = InMemoryChunkedUploadSizeAccumulator::new();
        let id = Uuid::new_v4();

        let size = accumulator.accumulate(id, 100);
        assert_eq!(100, size);

        accumulator.remove(id);
        let size = accumulator.get_total(id);
        assert_eq!(None, size);
    }
}
