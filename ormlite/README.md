# Development

Run the derive model code:

    cargo run --bin simple --features runtime-tokio-rustls,sqlite

Run test code:

    cargo test --features runtime-tokio-rustls,sqlite

Run tests themselves


### Workflow

Try to build and compile using the derive macros.

    just run --bin plain
    
Copy and paste into expanded.rs if you need to see details about the expanded code.

    cargo expand --bin plain | pbcopy
