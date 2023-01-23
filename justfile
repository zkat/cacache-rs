# List available just recipes
@help:
    just -l

# Run tests on both runtimes with cargo nextest
@test:
    echo "----------\nasync-std:\n"
    cargo nextest run
    echo "\n----------\ntokio:\n"
    cargo nextest run --no-default-features --features tokio-runtime

# Run benchmarks with `cargo bench`
@bench:
    echo "----------\nasync-std:\n"
    cargo bench
    echo "\n----------\ntokio:\n"
    cargo bench --no-default-features --features tokio-runtime

# Run benchmarks with `cargo criterion`
@criterion:
    echo "----------\nasync-std:\n"
    cargo criterion
    echo "\n----------\ntokio:\n"
    cargo criterion --no-default-features --features tokio-runtime

# Generate a changelog with git-cliff
changelog TAG:
    git-cliff --prepend CHANGELOG.md -u --tag {{TAG}}

# Prepare a release
release *args:
    cargo release --workspace {{args}}

# Install workspace tools
@install-tools:
    cargo install cargo-nextest
    cargo install cargo-release
    cargo install git-cliff
    cargo install cargo-criterion

# Lint and automatically fix what we can fix
@lint:
    cargo clippy --fix --allow-dirty --allow-staged
    cargo fmt
