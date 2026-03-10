use std::time::Duration;
use chrono::TimeDelta;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, warn};

pub trait Cache {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    /// Cache a value, ttl is in seconds
    fn set<T: Serialize>(&self, key: &str, value: T, ttl: Option<Duration>);
}

#[derive(Debug, Clone)]
pub struct NoOpCache;

impl Cache for NoOpCache {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        None
    }

    fn set<T: Serialize>(&self, key: &str, value: T, ttl: Option<Duration>) {
        // do nothing
    }
}

#[derive(Debug, Clone)]
pub struct MokaCache {
    cache: moka::sync::Cache<String, String>,
}

impl MokaCache {
    pub fn new() -> Self {
        Self {
            cache: moka::sync::Cache::new(10_000),
        }
    }
}

impl Cache for MokaCache {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let serialized_value = self.cache.get(&key.to_string());
        if let Some(serialized) = serialized_value {
            debug!("cache hit: {}", key);
            return serde_json::from_str(&serialized).ok();
        }

        None
    }

    fn set<T: Serialize>(&self, key: &str, value: T, ttl: Option<Duration>) {
        let serialized_value = match serde_json::to_string(&value) {
            Ok(serialized) => serialized,
            Err(err) => {
                warn!("unable to insert {} cache: {}", key, err);
                return;
            }
        };

        self.cache.insert(key.to_string(), serialized_value);
        debug!("inserted cache {} with expiration: {:?}", key, ttl);

        if let Some(ttl) = ttl {
            let this = self.clone();
            let key = key.to_string();
            let _ = std::thread::spawn(move || {
                std::thread::sleep(ttl);
                this.cache.invalidate(&key);
                debug!("cache {} expired after {} seconds", key, ttl.as_secs());
            });
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;


    const KEY: &str = "mikupush:example:key";
    const TTL_KEY: &str = "mikupush:example:ttl";
    const TTL: Duration = Duration::from_secs(5);

    #[test]
    fn test_moka_get() {
        let cache = MokaCache::new();
        cache.set(KEY, "example".to_string(), None);

        let cached: Option<String> = cache.get(KEY);
        assert_eq!(cached, Some("example".to_string()));
    }

    #[test]
    fn test_moka_ttl() {
        let cache = MokaCache::new();
        cache.set(TTL_KEY, "example".to_string(), Some(TTL));

        let cached: Option<String> = cache.get(TTL_KEY);
        assert_eq!(cached, Some("example".to_string()));

        std::thread::sleep(Duration::from_secs(6));

        let cached: Option<String> = cache.get(TTL_KEY);
        assert_eq!(cached, None);
    }

}
