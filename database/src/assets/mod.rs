mod schemas;
use diesel::prelude::*;

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = schemas::assets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Asset {
    pub id: i64,
    pub address: String,
    pub name: String,
    pub decimals: i32,
    pub symbol: String,
    pub chain_id: i64,
}

// implment query methods for Asset
impl Asset {
    pub fn find_by_address(
        conn: &mut PgConnection,
        asset_address: &str,
        asset_chain_id: i64,
    ) -> Option<Asset> {
        use schemas::assets::dsl::*;

        let result = assets
            .filter(address.eq(asset_address))
            .filter(chain_id.eq(asset_chain_id))
            .first::<Asset>(conn)
            .optional()
            .unwrap();

        result
    }

    pub fn find_by_id(conn: &mut PgConnection, asset_id: i64) -> Option<Asset> {
        use schemas::assets::dsl::*;

        let result = assets
            .filter(id.eq(asset_id))
            .first::<Asset>(conn)
            .optional()
            .unwrap();

        result
    }

    pub fn find_by_addresses(
        conn: &mut PgConnection,
        asset_addresses: Vec<String>,
        asset_chain_id: i64,
    ) -> Vec<Asset> {
        use schemas::assets::dsl::*;

        let result = assets
            .filter(address.eq_any(asset_addresses))
            .filter(chain_id.eq(asset_chain_id))
            .load::<Asset>(conn)
            .unwrap();

        result
    }
}
