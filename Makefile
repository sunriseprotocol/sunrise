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

.PHONY: build-eth
build-eth:
	cargo build --release --manifest-path node/dawn/Cargo.toml --features with-ethereum-compatibility

PHONY: run-eth
run-eth: 
	cargo run --manifest-path node/dawn/Cargo.toml --features with-ethereum-compatibility -- --dev -lruntime=debug -levm=debug --instant-sealing

.PHONY: test-eth
test-eth: 
	SKIP_WASM_BUILD= cargo test --manifest-path node/dawn/Cargo.toml test_evm_module --features with-ethereum-compatibility -p dawn-runtime

## Todo add in commands for overrides if needed
## cargo update && cargo update -p schnorrkel:0.9.2 --precise 0.9.1 && cargo update -p funty --precise 1.1.0

