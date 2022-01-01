


# Bump version. level=major,minor,patch
version level:
   git diff-index --exit-code HEAD > /dev/null || ! echo You have untracked changes. Commit your changes before bumping the version.

   cd ormlite-core && cargo set-version --bump {{level}}
   cd ormlite-core && cargo update # This bumps Cargo.lock

   cd ormlite-macro && cargo set-version --bump {{level}}
   cd ormlite-macro && cargo update # This bumps Cargo.lock

   cd ormlite && cargo set-version --bump {{level}}
   cd ormlite && cargo update # This bumps Cargo.lock

   VERSION=$(cd ormlite && rg  "version = \"([0-9.]+)\"" -or '$1' Cargo.toml | head -n1) && \
       git commit -am "Bump version {{level}} to $VERSION" && \
       git tag v$VERSION && \
       git push origin v$VERSION
   git push

publish:
   cd ormlite-core && cargo publish --features runtime-tokio-rustls,sqlite
   cd ormlite-macro && cargo publish --features runtime-tokio-rustls,sqlite
   cd ormlite && cargo publish --features runtime-tokio-rustls,sqlite,handwritten
