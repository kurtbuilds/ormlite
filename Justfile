test:
    just ormlite/test --features runtime-tokio-rustls,sqlite

# Bump version. level=major,minor,patch
version level:
   #!/bin/bash -euo pipefail
   function show() { dye -m -- "$@"; "$@"; }
   git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.

   echo $(dye -c INFO) Make sure that it builds first.
   show cd ormlite && cargo build --features runtime-tokio-rustls,sqlite

   show cargo set-version --bump {{ level }} --workspace
   export VERSION=$(cd ormlite && rg  "version = \"([0-9.]+)\"" -or '$1' Cargo.toml | head -n1)

   cd ormlite-macro && cargo add ormlite-core@$VERSION && cargo update
   cd ormlite && cargo add ormlite-core@$VERSION ormlite-macro@$VERSION && cargo update

   show git commit -am "Bump version {{level}} to $VERSION"
   show git tag v$VERSION
   show git push origin v$VERSION
   git push

publish:
   cd ormlite-core && cargo publish --features runtime-tokio-rustls,sqlite
   cd ormlite-macro && cargo publish --features runtime-tokio-rustls,sqlite
   cd ormlite && cargo publish --features runtime-tokio-rustls,sqlite
