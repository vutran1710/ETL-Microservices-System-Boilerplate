use sqlx::migrate::Migrator;
use sqlx::FromRow;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations/tier_1");

#[derive(Debug, FromRow)]
pub struct ExampleTable {
    pub id: String,
}
