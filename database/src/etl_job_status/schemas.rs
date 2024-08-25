diesel::table! {
    __etl_job_status (id) {
        id -> BigSerial,
        job_id -> VarChar,
        job_tier -> Integer,
        active_request -> Jsonb,
        received_at -> Timestamp,
        finished_at -> Nullable<Timestamp>,
        progress -> BigInt,
    }
}
