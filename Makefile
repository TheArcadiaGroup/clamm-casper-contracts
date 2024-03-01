PINNED_TOOLCHAIN := $(shell cat rust-toolchain)
prepare:
	rustup target add wasm32-unknown-unknown
	rustup component add clippy --toolchain ${PINNED_TOOLCHAIN}
	rustup component add rustfmt --toolchain ${PINNED_TOOLCHAIN}

build-all-contracts: build-test-math-session build-factory build-test-session build-test-callee build-router build-liquidity-session build-swap-session
	mkdir -p target
	cp tests/cep18.wasm tests/wasm/
	cp tests/wcspr-token.wasm tests/wasm/
	cp target/wasm32-unknown-unknown/release/*.wasm target/
	cp target/wasm32-unknown-unknown/release/*.wasm tests/wasm
	cp tests/get-session.wasm tests/wasm

build-router:
	cargo build --release -p router --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/router.wasm 2>/dev/null | true

build-test-math-session:
	mkdir -p tests/wasm
	cargo build --release -p test-math-session --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/test-math-session.wasm

build-liquidity-session:
	mkdir -p tests/wasm
	cargo build --release -p liquidity-session --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/liquidity-session.wasm

build-swap-session:
	mkdir -p tests/wasm
	cargo build --release -p swap-session --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/swap-session.wasm

build-test-session:
	mkdir -p tests/wasm
	cargo build --release -p test-session --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/test-session.wasm

build-test-callee:
	mkdir -p tests/wasm
	cargo build --release -p test-callee --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/test-callee.wasm
	
build-factory:
	mkdir -p tests/wasm
	cargo build --release -p factory --target wasm32-unknown-unknown
	wasm-strip target/wasm32-unknown-unknown/release/factory.wasm

test: build-all-contracts test-only

test-fast: build-all-contracts
	cd tests && cargo test

test-only: 
	cd tests && cargo test -- --test-threads 1

test-one: build-all-contracts
	cd tests && cargo test -p tests $(testname) -- --nocapture

clippy:
	cd math && cargo clippy --all-targets -- -D warnings
	cd common && cargo clippy --all-targets -- -D warnings
	cd tests && cargo clippy --all-targets -- -D warnings
	cd tests/test-math-session && cargo clippy --all-targets -- -D warnings
	cd tests/test-session && cargo clippy --all-targets -- -D warnings
	cd types && cargo clippy --all-targets -- -D warnings
	cd factory && cargo clippy --all-targets -- -D warnings
	cd liquidity-session && cargo clippy --all-targets -- -D warnings
	cd swap-session && cargo clippy --all-targets -- -D warnings
	cd tests/test-callee && cargo clippy --all-targets -- -D warnings

check-lint: clippy
	cd math && cargo fmt -- --check
	cd common && cargo fmt -- --check
	cd tests && cargo fmt -- --check
	cd tests/test-math-session && cargo fmt -- --check
	cd tests/test-session && cargo fmt -- --check
	cd types && cargo fmt -- --check
	cd factory && cargo fmt -- --check
	cd liquidity-session && cargo fmt -- --check
	cd swap-session && cargo fmt -- --check
	cd tests/test-callee && cargo fmt -- --check
	cd router/router && cargo fmt -- --check

lint: clippy
	cd math && cargo fmt
	cd common && cargo fmt
	cd tests && cargo fmt
	cd tests/test-math-session && cargo fmt
	cd tests/test-session && cargo fmt
	cd types && cargo fmt
	cd factory && cargo fmt
	cd liquidity-session && cargo fmt
	cd swap-session && cargo fmt
	cd tests/test-callee && cargo fmt
	cd router/router && cargo fmt

clean:
	rm -rf target
	cd math && cargo clean
	cd common && cargo clean
	cd tests && cargo clean
	cd tests/test-math-session && cargo clean
	cd tests/test-session && cargo clean
	cd types && cargo clean
	cd factory && cargo clean
	cd liquidity-session && cargo clean
	cd swap-session && cargo clean
	cd router/router && cargo clean
	cd tests/test-callee && cargo clean
