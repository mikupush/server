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