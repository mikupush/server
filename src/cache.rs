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

use std::collections::HashMap;
use std::ops::Add;
use std::sync::{Arc, LazyLock, LockResult, Mutex};
use std::time::Duration;
use chrono::{TimeDelta, Utc};
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

static CACHE_INSTANCE: LazyLock<MokaCache> = LazyLock::new(|| MokaCache::initialize());

#[derive(Debug, Clone)]
pub struct MokaCache {
    cache: moka::sync::Cache<String, String>,
    short_lived_keys: Arc<Mutex<HashMap<String, i64>>>
}

impl MokaCache {
    pub fn new() -> Self {
        Self {
            cache: moka::sync::Cache::new(10_000),
            short_lived_keys: Arc::new(Mutex::new(HashMap::new()))
        }
    }

    pub fn initialize() -> Self {
        let cache = MokaCache::new();
        // start invalidate caches job on singleton initialization
        cache.start_invalidate_expired();
        cache
    }

    pub fn current() -> Self {
        CACHE_INSTANCE.clone()
    }

    pub fn start_invalidate_expired(&self) {
        let this = self.clone();
        std::thread::spawn(move || {
            loop {
                let short_lived_keys_guard = this.short_lived_keys.lock();
                if let Err(err) = &short_lived_keys_guard {
                    warn!("unable to fetch cache short lived keys: {}", err);
                    return;
                }

                let now_ms = Utc::now().timestamp();
                let mut short_lived_keys_guard = short_lived_keys_guard.unwrap();
                let mut deleted_keys = Vec::<String>::new();

                debug!(expired_before = now_ms, "deleting expired cache keys");

                for (key, expires_at) in short_lived_keys_guard.iter() {
                    if *expires_at < now_ms {
                        debug!(key = key, expires_at = *expires_at, "cache key expired");
                        deleted_keys.push(key.clone());
                        this.cache.invalidate(key);
                    }
                }

                for key in deleted_keys {
                    let _ = short_lived_keys_guard.remove(&key);
                }

                std::thread::sleep(Duration::from_millis(1));
            }
        });
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
            let key = key.to_string();
            let expires_at = Utc::now().add(ttl);
            let expires_at_ms = expires_at.timestamp();
            let short_lived_keys_guard = self.short_lived_keys.lock();
            if let Err(err) = &short_lived_keys_guard {
                warn!("unable to add ttl to cache {}: {}", key, err);
                return;
            }

            let mut short_lived_keys_guard = short_lived_keys_guard.unwrap();
            let _ = short_lived_keys_guard.insert(key, expires_at_ms);
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
        cache.start_invalidate_expired();
        cache.set(TTL_KEY, "example".to_string(), Some(TTL));

        let cached: Option<String> = cache.get(TTL_KEY);
        assert_eq!(cached, Some("example".to_string()));

        std::thread::sleep(Duration::from_secs(6));

        let cached: Option<String> = cache.get(TTL_KEY);
        assert_eq!(cached, None);
    }

}
