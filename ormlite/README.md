# Development

Run the handwritten code:

    cargo run --bin handwritten --features handwritten,sqlite,runtime-tokio-rustls

Run the derive model code:

    cargo run --bin plain --features runtime-tokio-rustls,sqlite,handwritten

Run test code:

    cargo test --features runtime-tokio-rustls,sqlite,handwritten

Run tests themselves


### Workflow

Try to build and compile using the derive macros.

    just run --bin plain
    
Copy and paste into expanded.rs if you need to see details about the expanded code.

    cargo expand --bin plain | pbcopy
