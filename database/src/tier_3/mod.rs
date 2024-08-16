mod schemas;

use crate::OrderingID;
use chrono::NaiveDate;
use diesel::data_types::PgNumeric;
use diesel::prelude::*;
use serde::Deserialize;
use serde::Serialize;

#[derive(Queryable, Selectable)]
#[diesel(table_name = schemas::balance_per_date)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct BalancePerDate {
    pub user: String,
    pub balance: PgNumeric,
    pub date: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct QueryID {
    pub user: String,
    pub date: NaiveDate,
}

impl OrderingID for QueryID {}
