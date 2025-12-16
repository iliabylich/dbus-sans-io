build type *args:
    cargo build --bin {{type}} --features {{type}} {{args}}
run type *args:
    cargo run --bin {{type}} --features {{type}} {{args}}
clippy type:
    cargo clippy --features {{type}}

blocking:
    @just run blocking
poll:
    @just run poll
io-uring:
    cargo run --bin io-uring --features io-uring-with-dep

clippy-all:
    @just clippy blocking
    @just clippy poll
    cargo clippy --features io-uring-with-dep

build-release:
    @just build blocking --release
    @just build poll --release
    cargo build --bin io-uring --features io-uring-with-dep --release
    ls -l target/release/ | grep -E "blocking|poll|io-uring" | grep -vF ".d"
