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
    use crate::database::DbPool;

    pub fn create_test_database_connection() -> DbPool {
        let connection_string = "postgres://miku_push:miku_push@localhost:5432/miku_push";
        let manager = ConnectionManager::<PgConnection>::new(connection_string);
        Pool::builder().build(manager).unwrap()
    }
}
