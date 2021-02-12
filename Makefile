.PHONY: init
init:
	./scripts/init.sh

.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --all

.PHONY: buildrun
buildrun:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-06 cargo build --release; ./target/release/sunrise purge-chain -y --chain node/sunrise/chain_spec/local.json; ./target/release/sunrise --alice --chain node/sunrise/chain_spec/local.json

.PHONY: run
run:
	./target/release/sunrise purge-chain -y --chain node/sunrise/chain_spec/local.json; ./target/release/sunrise --alice --chain node/sunrise/chain_spec/local.json

.PHONY: build
build:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-06 cargo build --release


.PHONY: rundev
rundev:
	./target/release/sunrise purge-chain -y --dev; ./target/release/sunrise --alice --dev
