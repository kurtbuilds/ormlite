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
   #!/bin/bash -euo pipefail
   git diff-index --exit-code HEAD > /dev/null || { echo "You have uncommitted changes. Commit them before bumping the version."; exit 1; }
   (cd ormlite && cargo build --features runtime-tokio-rustls,sqlite)
   CURRENT=$(grep -E '^version = ' Cargo.toml | head -n1 | sed -E 's/version = "(.*)"/\1/')
   IFS=. read -r MAJOR MINOR PATCH <<< "$CURRENT"
   case "{{level}}" in
     major) MAJOR=$((MAJOR+1)); MINOR=0; PATCH=0 ;;
     minor) MINOR=$((MINOR+1)); PATCH=0 ;;
     patch) PATCH=$((PATCH+1)) ;;
     *) echo "level must be major|minor|patch"; exit 1 ;;
   esac
   NEW="$MAJOR.$MINOR.$PATCH"
   echo "Bumping $CURRENT -> $NEW"
   sed -i.bak -E "s/^version = \".*\"/version = \"$NEW\"/" Cargo.toml
   rm Cargo.toml.bak
   cargo check --workspace > /dev/null
   git commit -am "Bump version {{level}}"
   git tag v$NEW
   git push
   git push --tags

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
