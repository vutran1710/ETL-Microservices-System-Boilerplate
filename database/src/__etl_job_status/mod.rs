use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde_json::Value;
mod schemas;

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = schemas::__etl_job_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EtlJobStatus {
    pub id: i64,
    pub job_id: String,
    pub active_request: Value,
    pub received_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
}

impl EtlJobStatus {
    pub fn find_all_unfinished_jobs(
        conn: &mut PgConnection,
        etl_job_id: &str,
    ) -> Result<Vec<Self>, diesel::result::Error> {
        use schemas::__etl_job_status::dsl::*;

        __etl_job_status
            .filter(job_id.eq(etl_job_id))
            .filter(finished_at.is_null())
            .order(received_at.desc())
            .load::<EtlJobStatus>(conn)
    }

    pub fn save(&self, conn: &mut PgConnection) -> Result<Self, diesel::result::Error> {
        use schemas::__etl_job_status::dsl::*;

        let values = vec![(
            job_id.eq(&self.job_id),
            active_request.eq(&self.active_request),
        )];

        diesel::insert_into(__etl_job_status)
            .values(&values)
            .get_result(conn)
    }

    pub fn set_job_as_finished(
        conn: &mut PgConnection,
        job_pk: i64,
    ) -> Result<usize, diesel::result::Error> {
        use schemas::__etl_job_status::dsl::*;

        diesel::update(__etl_job_status.filter(id.eq(job_pk)))
            .set(finished_at.eq(Some(chrono::Utc::now().naive_utc())))
            .execute(conn)
    }
}
