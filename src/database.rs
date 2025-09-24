/// Copyright 2025 Miku Push! Team
///
/// Licensed under the Apache License, Version 2.0 (the "License");
/// you may not use this file except in compliance with the License.
/// You may obtain a copy of the License at
///
///     http://www.apache.org/licenses/LICENSE-2.0
///
/// Unless required by applicable law or agreed to in writing, software
/// distributed under the License is distributed on an "AS IS" BASIS,
/// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
/// See the License for the specific language governing permissions and
/// limitations under the License.

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use log::error;
use crate::config::Settings;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_database_connection(settings: Settings) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(settings.database.url());
    Pool::builder().build(manager).unwrap_or_else(|err| {
        error!("Error creating database connection pool: {}", err);
        panic!("Error creating pool: {}", err)
    })
}

#[cfg(test)]
pub mod tests {
    use diesel::PgConnection;
    use diesel::r2d2::ConnectionManager;
    use r2d2::Pool;
    use crate::config::Settings;
    use crate::database::DbPool;

    pub fn create_test_database_connection() -> DbPool {
        let settings = Settings::load();
        let manager = ConnectionManager::<PgConnection>::new(settings.database.url());
        Pool::builder().build(manager).unwrap()
    }
}
