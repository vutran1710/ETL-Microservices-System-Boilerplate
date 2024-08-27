# ETL-Microservice-System

## Development
- Create new ETL app with Make:
```bash
$ make create-etl name={app-name} tables={table1,table2}
```

- After creating the app, you can find the app in `crates/{app-name}/lib.rs` directory that looks like this:
```rust
struct SomeState {
    state: std::collections::HashMap<String, Vec<i64>>,
}

fn handle_data(
    table: Table,
    range: RangeQuery,
    source: &mut PgConnection,
    sink: &mut PgConnection,
    state: &mut SomeState,
) -> eyre::Result<Option<(Table, RangeQuery)>> {
    log::info!("Processing changes for table: {:?}", table);

    todo!("Implement here")
}

create_etl_job!(
    id => "{app-name}",
    state => SomeState,
    handle_data
);
```

Implement the `handle_data` function to process the data.

- The `table` is the table name that you receive from the Message Queue.
- The `range` is the range query that you receive from the Message Queue. It tells the changes happened for the table in the given range.
- For sink tables, you will need to import the tables you specified in the `tables` argument in the `create-etl` command. All the tables should be found in `database` crates and automatically exported for usage in your app.
- Sink the data to the sink database using the `sink` connection. The `source` connection is used to query the data from the source database.
- The `state` is used to store the state of the processing. For example, if you want to store the last processed id of a table, you can store it in the `state` struct.


## Build
- Build the **etl-app** using `cargo build --release -F  {feature-name}`

#### app env
```rust
struct Args {
    #[arg(
        long,
        env = "ETL_SOURCE",
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    source: String,

    #[arg(
        long,
        env = "ETL_SINK",
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    sink: String,

    #[arg(
        long,
        env = "ETL_JOB_MANAGER",
        default_value = "postgres://postgres:postgres@localhost:5432/postgres"
    )]
    job_manager: String,

    #[arg(long, env = "ETL_SERVER_PORT", default_value = "8080")]
    port: u16,
}
```

#### rabbitmq env
```rust
pub struct Args {
    #[arg(long, env = "RABBITMQ_SOURCE_QUEUE")]
    pub source_queue: String,
    #[arg(long, env = "RABBITMQ_SINK_QUEUE")]
    pub sink_queue: String,
    #[arg(long, env = "RABBITMQ_HOST", default_value = "localhost")]
    pub host: String,
    #[arg(long, env = "RABBITMQ_USERNAME", default_value = "guest")]
    pub username: String,
    #[arg(long, env = "RABBITMQ_PASSWORD", default_value = "guest")]
    pub password: String,
}
```

## Command to run
```rust
$ cargo run -p etl-app -F {app-name}
```

- When run, application has a api server that user can send manual processing request at `http://{host}:{port}/process`. This api accepts POST only.
- Checkout `libs/common/messages` for the structure of the payload.
- Example query for POST payload:
```json
{
  "DataStoreUpdated": {
    "table": "actions",
    "range": {
        "range": {
          "numeric": {
              "from": 1,
              "to": 10
          }
        },
        "filters": {
          "user": "abcde"
        }
    }
  }
}
```
