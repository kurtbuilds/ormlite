set dotenv-load := true

test:
    just ormlite/test --features runtime-tokio-rustls,sqlite

# Bump version. level=major,minor,patch
version level:
   #!/bin/bash -euxo pipefail
   git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version. || exit 1

   echo $(dye -c INFO) Make sure that it builds first.
   (cd ormlite && cargo build --features runtime-tokio-rustls,sqlite)

   cargo set-version --bump {{ level }} --workspace
   VERSION=$(toml get ormlite/Cargo.toml package.version)

   toml set macro/Cargo.toml dependencies.ormlite-core.version $VERSION
   (cd macro && cargo update)
   toml set ormlite/Cargo.toml dependencies.ormlite-core.version $VERSION
   toml set ormlite/Cargo.toml dependencies.ormlite-macro.version $VERSION
   (cd ormlite && cargo update)

   git commit -am "Bump version {{level}} to $VERSION"
   git tag v$VERSION
   git push

publish:
   cd ormlite-core && cargo publish --features runtime-tokio-rustls,sqlite
   @echo $(dye -c INFO) Need to sleep so that crates.io has time to update.
   sleep 5
   cd ormlite-macro && cargo publish --features runtime-tokio-rustls,sqlite
   @echo $(dye -c INFO) Need to sleep so that crates.io has time to update.
   sleep 5
   cd ormlite && cargo publish --features runtime-tokio-rustls,sqlite

doc:
   cd ormlite && cargo doc --features runtime-tokio-rustls,sqlite --open -p ormlite --no-deps
