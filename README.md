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
