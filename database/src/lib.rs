mod interfaces;

pub use interfaces::*;

#[cfg(feature = "tier_1")]
pub mod tier_1;

#[cfg(feature = "tier_2")]
pub mod tier_2;

pub use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn create_pg_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
