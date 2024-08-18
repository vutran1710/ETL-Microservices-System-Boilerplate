mod query_interfaces;
pub use query_interfaces::*;
use serde::Deserialize;
use serde::Serialize;
use strum::Display;
use strum::EnumString;

#[cfg(feature = "tier_1")]
pub mod tier_1;

#[cfg(feature = "tier_2")]
pub mod tier_2;

#[cfg(feature = "tier_3")]
pub mod tier_3;

pub use diesel::pg::PgConnection;
use diesel::prelude::*;

pub fn create_pg_connection(database_url: &str) -> PgConnection {
    PgConnection::establish(database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display, Hash)]
pub enum Table {
    #[cfg(feature = "tier_1")]
    Tier1(tier_1::Table),

    #[cfg(feature = "tier_2")]
    Tier2(tier_2::Table),

    #[cfg(feature = "tier_3")]
    Tier3(tier_3::Table),
}
