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

# Bump version. level=major,minor,patch
version level:
   #!/bin/bash -euxo pipefail
   git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version. || exit 1

   echo $(dye -c INFO) Make sure that it builds first.
   (cd ormlite && cargo build --features runtime-tokio-rustls,sqlite)

   cargo set-version --bump {{ level }} --workspace
   VERSION=$(rg -om1 "version = \"(.*)\"" --replace '$1' ormlite/Cargo.toml)
   git commit -am "Bump version {{level}}"
   git tag v$VERSION
   git push
   git push --tags

patch:
    just version patch
    just publish

publish:
   cd attr && cargo publish
   cd core && cargo publish --features sqlite,postgres,mysql,runtime-tokio-rustls
   cd macro && cargo publish --features sqlite,postgres,mysql,runtime-tokio-rustls
   cd ormlite && cargo publish --features sqlite,postgres,mysql
   cd cli && cargo publish

doc:
   cd ormlite && RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --all-features --open -p ormlite --no-deps

install:
    @just cli/install

postgres *ARGS:
    @just ormlite/postgres $ARGS
