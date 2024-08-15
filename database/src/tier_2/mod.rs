use sqlx::migrate::Migrator;

pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations/tier_2");

pub struct ExampleTable2 {}
