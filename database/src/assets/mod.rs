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

impl Asset {
    pub fn save(&self, conn: &mut PgConnection) -> eyre::Result<Asset> {
        use schemas::assets::dsl::*;

        let values = vec![(
            address.eq(&self.address),
            name.eq(&self.name),
            decimals.eq(&self.decimals),
            symbol.eq(&self.symbol),
            chain_id.eq(&self.chain_id),
        )];

        let result = diesel::insert_into(assets)
            .values(&values)
            .on_conflict((address, chain_id))
            .do_nothing()
            .get_result(conn)?;

        Ok(result)
    }

    pub fn save_mut(&mut self, conn: &mut PgConnection) -> eyre::Result<()> {
        use schemas::assets::dsl::*;

        let values = vec![(
            address.eq(&self.address),
            name.eq(&self.name),
            decimals.eq(&self.decimals),
            symbol.eq(&self.symbol),
            chain_id.eq(&self.chain_id),
        )];

        let result: Asset = diesel::insert_into(assets)
            .values(&values)
            .on_conflict((address, chain_id))
            .do_nothing()
            .get_result(conn)?;

        self.id = result.id;
        Ok(())
    }

    pub fn find_by_address(
        conn: &mut PgConnection,
        asset_address: &str,
        asset_chain_id: i64,
    ) -> eyre::Result<Option<Asset>> {
        use schemas::assets::dsl::*;

        let result = assets
            .filter(address.eq(asset_address))
            .filter(chain_id.eq(asset_chain_id))
            .first::<Asset>(conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_id(conn: &mut PgConnection, asset_id: i64) -> eyre::Result<Option<Asset>> {
        use schemas::assets::dsl::*;

        let result = assets
            .filter(id.eq(asset_id))
            .first::<Asset>(conn)
            .optional()?;

        Ok(result)
    }

    pub fn find_by_addresses(
        conn: &mut PgConnection,
        asset_addresses: Vec<String>,
        asset_chain_id: i64,
    ) -> eyre::Result<Vec<Asset>> {
        use schemas::assets::dsl::*;

        let result = assets
            .filter(address.eq_any(asset_addresses))
            .filter(chain_id.eq(asset_chain_id))
            .load::<Asset>(conn)?;

        Ok(result)
    }
}
