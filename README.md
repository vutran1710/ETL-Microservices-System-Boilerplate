# ETL-Microservice-System

## Introduction
...

## Development
- Create new ETL app with Make:
```bash
$ make create-etl name={app-name} tables={table1,table2}
```


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
