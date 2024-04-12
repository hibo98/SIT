use diesel::{r2d2::{ConnectionManager, Pool}, Connection, SqliteConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use self::task::TaskManager;

mod model;
mod schema;
mod task;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct Database {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Database {
    pub fn establish_connection() -> Database {
        let database_url = "sqlite://./db_client.db";

        SqliteConnection::establish(database_url)
            .unwrap_or_else(|_| panic!("Error connection to {database_url}"))
            .run_pending_migrations(MIGRATIONS)
            .expect("Migrations failed");

        let manager = ConnectionManager::<SqliteConnection>::new(database_url);

        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool");

        Database {
            pool
        }
    }

    pub fn task_manager(&self) -> TaskManager {
        TaskManager::new(self.pool.clone())
    }
}
