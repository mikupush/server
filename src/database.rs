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

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::error;
use crate::config::Settings;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub fn create_database_connection(settings: Settings) -> DbPool {
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

#[cfg(test)]
pub mod tests {
    use crate::config::Settings;
    use crate::database::{create_database_connection, DbPool};

    pub fn create_test_database_connection() -> DbPool {
        let settings = Settings::load();
        create_database_connection(settings)
    }
}