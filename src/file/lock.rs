use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ChunkedReaderLock {
    locks: Arc<AtomicU64>
}

impl ChunkedReaderLock {
    pub fn new() -> Self {
        Self { locks: Arc::new(AtomicU64::new(0)) }
    }

    pub fn lock(&self) {
        self.locks.fetch_add(1, Ordering::Relaxed);
    }

    pub fn unlock(&self) {
        let locks = self.locks.load(Ordering::Relaxed);

        if locks > 0 {
            self.locks.fetch_sub(1, Ordering::Relaxed);
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locks.load(Ordering::Relaxed) > 0
    }
}

pub trait ChunkedReaderLocker {
    fn lock_for(&self, id: &Uuid) -> Result<(), String>;
    fn unlock_for(&self, id: &Uuid) -> Result<(), String>;
    fn is_locked(&self, id: &Uuid) -> Result<bool, String>;
}

static CHUNKED_READER_LOCKER: OnceLock<InMemoryChunkedReaderLocker> = OnceLock::new();

#[derive(Clone)]
pub struct InMemoryChunkedReaderLocker {
    locks: Arc<Mutex<HashMap<Uuid, ChunkedReaderLock>>>
}

impl InMemoryChunkedReaderLocker {
    pub fn new() -> Self {
        Self { locks: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub fn get_instance() -> Self {
        CHUNKED_READER_LOCKER.get_or_init(|| Self::new()).clone()
    }
}

impl ChunkedReaderLocker for InMemoryChunkedReaderLocker {
    fn lock_for(&self, id: &Uuid) -> Result<(), String> {
        debug!(upload = ?id, "locking chunked read");
        let mut locks = self.locks.lock()
            .map_err(|_| format!("failed to lock the reader locks for {}", id))?;
        let lock = locks.get(&id)
            .map(|lock| lock.clone())
            .unwrap_or_else(|| ChunkedReaderLock::new());

        lock.lock();
        locks.insert(*id, lock);

        Ok(())
    }

    fn unlock_for(&self, id: &Uuid) -> Result<(), String> {
        debug!(upload = ?id, "unlocking chunked read");
        let mut locks = self.locks.lock()
            .map_err(|_| format!("failed to lock the reader locks for {}", id))?;
        let lock = locks.get(&id).map(|lock| lock.clone());

        if let Some(lock) = lock {
            lock.unlock();
            locks.insert(*id, lock);
        }

        Ok(())
    }

    fn is_locked(&self, id: &Uuid) -> Result<bool, String> {
        let locks = self.locks.lock()
            .map_err(|_| format!("failed to lock the reader locks for {}", id))?;
        let is_locked = locks.get(&id)
            .map(|lock| lock.is_locked())
            .unwrap_or(false);

        Ok(is_locked)
    }
}

#[cfg(test)]
mod tests {
    use rand::random;
    use super::*;

    #[test]
    fn test_lock() {
        let lock = ChunkedReaderLock::new();
        lock.lock();

        assert_eq!(true, lock.is_locked())
    }

    #[test]
    fn test_unlock() {
        let lock = ChunkedReaderLock::new();

        lock.lock();
        assert_eq!(true, lock.is_locked());

        lock.unlock();
        assert_eq!(false, lock.is_locked());
    }

    #[test]
    fn test_unlock_multiple_threads() {
        let lock = ChunkedReaderLock::new();
        let mut handles = Vec::new();

        for i in 0..5 {
            let lock = lock.clone();
            handles.push(std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(random::<u64>() % 100));
                lock.lock();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(true, lock.is_locked());

        let mut handles = Vec::new();

        for i in 0..5 {
            let lock = lock.clone();
            handles.push(std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(random::<u64>() % 100));
                lock.unlock();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(false, lock.is_locked());
    }

    #[test]
    fn test_unlock_multiple_threads_concurrent() {
        let lock = ChunkedReaderLock::new();
        let mut handles = Vec::new();

        for i in 0..5 {
            let lock = lock.clone();
            handles.push(std::thread::spawn(move || {
                lock.lock();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(true, lock.is_locked());

        let mut handles = Vec::new();

        for i in 0..15 {
            let lock = lock.clone();
            handles.push(std::thread::spawn(move || {
                lock.unlock();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(false, lock.is_locked());
    }

    #[test]
    fn test_inmemory_locker_lock() {
        let locker = InMemoryChunkedReaderLocker::new();
        let id = Uuid::new_v4();

        locker.lock_for(&id).unwrap();
        assert_eq!(true, locker.is_locked(&id).unwrap());
    }

    #[test]
    fn test_inmemory_locker_unlock() {
        let locker = InMemoryChunkedReaderLocker::new();
        let id = Uuid::new_v4();

        locker.lock_for(&id).unwrap();
        assert_eq!(true, locker.is_locked(&id).unwrap());

        locker.unlock_for(&id).unwrap();
        assert_eq!(false, locker.is_locked(&id).unwrap());
    }
}
