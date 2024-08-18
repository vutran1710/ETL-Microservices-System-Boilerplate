# ETL-Microservice-System

## Introduction
...

## Development
- Create a new etl crate in `crates/` using `cargo new --lib crates/etl-lib-name`
- Add the etl crate to the workspace in `Cargo.toml` in `members` array and in `dependencies` section
- Add the etl crate to the **etl-app** in Cargo with proper feature name
- Import command from the etl crate in the **etl-app** and use it in the main function


## Build
- Build the **etl-app** using `cargo build --release -F  {feature-name}`
- Replace `{feature-name}` with the feature name of the etl crate you want to build in `main.rs`

## Test with Example

The following example runs crates/example requires:
- RabbitMQ server running on `localhost:5672`
- PostgreSQL server running on `localhost:5432`, with 2 tables: `transactions`(for **Tier1**) and `buy_sell`(for **Tier2**)
- Prepare required environment variables, including

#### app env
```rust
struct Args {
    #[arg(long, env = "ETL_SOURCE")] // Postgres database url of source table (eg: transactions)
    source: String,
    #[arg(long, env = "ETL_SINK")] // Postgres database url of sink table (eg: buy_sell)
    sink: String,
    #[arg(long, env = "ETL_SERVER_PORT")]
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
$ cargo run -p etl-app -F example_with_rabbitmq
```
