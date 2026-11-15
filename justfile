# GitHub ReDesktop — task runner (https://just.systems)

name := 'github-redesktop'
appid := 'io.github.kisakay.GithubReDesktop'

rootdir := ''
prefix := '/usr'
cargo-target-dir := env('CARGO_TARGET_DIR', 'target')

base-dir := absolute_path(clean(rootdir / prefix))
bin-src := cargo-target-dir / 'release' / name
bin-dst := base-dir / 'bin' / name
desktop-dst := base-dir / 'share' / 'applications' / appid + '.desktop'
metainfo-dst := base-dir / 'share' / 'metainfo' / appid + '.metainfo.xml'
icon-dst := base-dir / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps' / appid + '.svg'

# Build in release mode
default: build-release

# Compile with the debug profile
build-debug *args:
    cargo build --locked {{args}}

# Compile with the release profile
build-release *args: (build-debug '--release' args)

# Run the application
run *args:
    env RUST_BACKTRACE=full cargo run --release --locked {{args}}

# Run the test-suite
test *args:
    cargo test --locked {{args}}

# Lint with clippy
check *args:
    cargo clippy --all-targets --locked {{args}} -- -D warnings

# Format the code
fmt:
    cargo fmt --all

# Install into the prefix (`sudo just install` or `just rootdir=pkg install`)
install:
    install -Dm0755 {{bin-src}} {{bin-dst}}
    install -Dm0644 resources/{{appid}}.desktop {{desktop-dst}}
    install -Dm0644 resources/{{appid}}.metainfo.xml {{metainfo-dst}}
    install -Dm0644 resources/icons/hicolor/scalable/apps/{{appid}}.svg {{icon-dst}}

# Remove installed files
uninstall:
    rm -f {{bin-dst}} {{desktop-dst}} {{metainfo-dst}} {{icon-dst}}

# Build a portable tarball in dist/ from an existing release build
tarball arch=`uname -m`:
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
    dir="dist/{{name}}-${version}-linux-{{arch}}"
    mkdir -p "$dir"
    just rootdir="$dir" prefix=/ install
    cp README.md LICENSE packaging/install.sh "$dir/"
    tar -C dist -czf "$dir.tar.gz" "$(basename "$dir")"
    echo "$dir.tar.gz"
