
use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager, Pool};
use dotenvy::dotenv;
use std::env;



pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")

}




