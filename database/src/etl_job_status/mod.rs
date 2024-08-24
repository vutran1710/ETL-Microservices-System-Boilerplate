use std::ops::DerefMut;
use std::sync::Arc;
use std::sync::Mutex;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde_json::Value;
mod schemas;

// Database tables are defined here ------------------------------------------------------
#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = schemas::etl_job_status)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct EtlJobStatus {
    pub id: i64,
    pub job_id: String,
    pub active_request: Value,
    pub received_at: NaiveDateTime,
    pub finished_at: Option<NaiveDateTime>,
    pub progress: i64,
}

#[derive(Clone)]
pub struct EtlJobManager {
    job_id: String,
    conn: Arc<Mutex<PgConnection>>,
    active_jobs: Arc<Mutex<Vec<EtlJobStatus>>>,
}

impl EtlJobManager {
    fn new(url: &str, id: &str) -> Self {
        let conn =
            PgConnection::establish(url).unwrap_or_else(|_| panic!("Error connecting to {}", url));

        Self {
            conn: Arc::new(Mutex::new(conn)),
            job_id: id.to_string(),
            active_jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn latest_active_jobs(&self) -> Vec<EtlJobStatus> {
        use schemas::etl_job_status::dsl::*;

        let mut conn = self.conn.lock().unwrap();
        let result = etl_job_status
            .filter(job_id.eq(&self.job_id))
            .filter(finished_at.is_null())
            .order(received_at.desc())
            .get_results(conn.deref_mut())
            .unwrap();

        result
    }

    pub fn initialize(job_id: &str, db: &str) -> Self {
        let manager = EtlJobManager::new(db, job_id);
        let active_jobs_from_db = manager.latest_active_jobs();
        let mut active_jobs = manager.active_jobs.lock().unwrap();
        active_jobs.extend(active_jobs_from_db);
        drop(active_jobs);
        manager
    }

    pub fn add_new_job(&self, new_request: Value) -> eyre::Result<EtlJobStatus> {
        use schemas::etl_job_status::dsl::*;

        let mut conn = self.conn.lock().unwrap();
        let rows = vec![(
            job_id.eq(&self.job_id),
            active_request.eq(new_request),
            received_at.eq(chrono::Utc::now().naive_utc()),
            progress.eq(-1),
        )];

        let inserted_job = diesel::insert_into(etl_job_status)
            .values(&rows)
            .get_result::<EtlJobStatus>(conn.deref_mut())?;

        log::info!("Inserted new job with id {}", inserted_job.id);

        let mut active_jobs = self.active_jobs.lock().unwrap();
        active_jobs.push(inserted_job.clone());
        Ok(inserted_job)
    }

    pub fn update_progress(&self, updated_progress: i64, job_index: usize) -> eyre::Result<()> {
        use schemas::etl_job_status::dsl::*;

        let active_jobs = self.active_jobs.lock().unwrap();

        if let Some(job) = active_jobs.get(job_index) {
            let mut conn = self.conn.lock().unwrap();
            diesel::update(etl_job_status.find(job.id))
                .set(progress.eq(updated_progress))
                .execute(conn.deref_mut())?;
            log::info!("Updated progress for job {}", job.id);
            Ok(())
        } else {
            return Err(eyre::eyre!("Job not found"));
        }
    }

    pub fn set_finished_at(&self, job_index: usize) -> eyre::Result<()> {
        use schemas::etl_job_status::dsl::*;

        let mut active_jobs = self.active_jobs.lock().unwrap();

        if let Some(job) = active_jobs.get(job_index) {
            let mut conn = self.conn.lock().unwrap();
            diesel::update(etl_job_status.find(job.id))
                .set(finished_at.eq(Some(chrono::Utc::now().naive_utc())))
                .execute(conn.deref_mut())?;
            log::info!("Job {} is finished", job.id);
            active_jobs.remove(job_index);
            Ok(())
        } else {
            return Err(eyre::eyre!("Job not found"));
        }
    }
}
