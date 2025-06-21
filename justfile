dev:
    just fmt
    just lint
    just test

fmt *ARGS:
    cargo fmt --all {{ARGS}}

lint *ARGS:
    cargo clippy --all-targets --all-features {{ARGS}}

test *ARGS:
    cargo test --all-features {{ARGS}}

doc *ARGS:
    RUSTDOCFLAGS="--cfg docsrs" cargo +nightly doc --open --no-deps --all-features {{ARGS}}

ci:
    just fmt --check
    just lint -- -D warnings
    just test

sync-version:
    cargo set-version -p bool-logic "0.3.1-dev"
    cargo set-version -p libc-cfg   "0.3.1-dev"
