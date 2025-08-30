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
