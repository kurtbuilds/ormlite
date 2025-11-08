set dotenv-load
set positional-arguments
set export

check:
    cargo check

test:
    just attr/test
    just core/test
    just macro/test
    just ormlite/test
    just cli/build

patch:
    just version patch
    just publish

publish:
   cargo publish --workspace --features sqlite,postgres,mysql,runtime-tokio-rustls

doc:
   cd ormlite && RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --open -p ormlite --no-deps

install:
    @just cli/install

postgres *ARGS:
    @just ormlite/postgres $ARGS
