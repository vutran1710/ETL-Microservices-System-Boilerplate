mod __etl_job_status;
mod assets;
mod query_interfaces;

pub use __etl_job_status::EtlJobManager;
pub use assets::Asset;
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

#[derive(EnumString, Debug, Clone, PartialEq, Eq, Display, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Table {
    #[cfg(feature = "tier_1")]
    Tier1(tier_1::Table),

    #[cfg(feature = "tier_2")]
    Tier2(tier_2::Table),

    #[cfg(feature = "tier_3")]
    Tier3(tier_3::Table),
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "tier_1")]
    fn test_view_table_enum() {
        env_logger::try_init().ok();
        let ex_table = super::Table::Tier1(crate::tier_1::Table::Actions);
        let table_str = serde_json::to_string(&ex_table).unwrap();
        log::info!("table_str: {}", table_str);
        assert_eq!(table_str, "\"actions\"");
    }
}
