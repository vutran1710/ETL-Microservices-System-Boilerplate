#[cfg(feature = "interfaces")]
pub mod interfaces;

#[cfg(feature = "tier_1")]
pub mod tier_1;

#[cfg(feature = "tier_2")]
pub mod tier_2;

use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
