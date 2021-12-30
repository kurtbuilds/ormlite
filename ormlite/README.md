# Development

Run test code:
    
    just run --bin plain --features runtime-tokio-rustls,sqlite,handwritten

### Workflow

Try to build and compile using the derive macros.

    just run --bin plain
    
Copy and paste into expanded.rs if you need to see details about the expanded code.

    cargo expand --bin plain | pbcopy
