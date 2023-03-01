set dotenv-load := true

test:
    just attr/test
    just core/test
    just macro/test
    just ormlite/test
    just cli/build

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
   cd attr && cargo publish
   cd core && cargo publish --features sqlite,postgres,mysql,runtime-tokio-rustls
   cd macro && cargo publish --features sqlite,postgres,mysql,runtime-tokio-rustls
   cd ormlite && cargo publish --features sqlite,postgres,mysql
   cd cli && cargo publish

doc:
   cd ormlite && cargo doc --all-features --open -p ormlite --no-deps

install:
    @just cli/install
