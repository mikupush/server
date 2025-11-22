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

use std::sync::OnceLock;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::error;
use crate::config::Settings;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
static DB_POOL: OnceLock<DbPool> = OnceLock::new();

pub fn setup_database_connection(settings: &Settings) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(settings.database.url());
    let pool = Pool::builder().build(manager).unwrap_or_else(|err| {
        error!("Error creating database connection pool: {}", err);
        panic!("Error creating pool: {}", err)
    });

    let mut connection = pool.get().expect("Error connecting to database");

    connection.run_pending_migrations(MIGRATIONS)
        .expect("Error running migrations");

    pool
}

pub fn get_database_connection(settings: Settings) -> DbPool {
    if let Some(pool) = DB_POOL.get() {
        return pool.clone()
    }

    let pool = setup_database_connection(&settings);
    DB_POOL.set(pool.clone()).expect("database connection pool already set");
    pool
}

#[cfg(test)]
pub mod tests {
    use std::sync::OnceLock;
    use crate::config::Settings;
    use crate::database::{setup_database_connection, get_database_connection, DbPool};

    static TEST_DB_POOL: OnceLock<DbPool> = OnceLock::new();

    pub fn get_test_database_connection() -> DbPool {
        if let Some(pool) = TEST_DB_POOL.get() {
            return pool.clone();
        }

        let settings = Settings::load();
        let pool = setup_database_connection(&settings);
        TEST_DB_POOL.set(pool.clone())
            .expect("database connection pool already set");
        pool
    }
}
