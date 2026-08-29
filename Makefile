.PHONY: build run-sim test lint cross-compile-arm clean

build:
	cargo build --workspace --release

run-sim:
	cargo run --bin mesh-sim

test:
	cargo test --workspace

lint:
	cargo fmt --check
	cargo clippy --workspace -- -D warnings

cross-compile-arm:
	# Requires rustup target add aarch64-unknown-linux-gnu and cross-compiler
	cargo build --target aarch64-unknown-linux-gnu --release

clean:
	cargo clean
