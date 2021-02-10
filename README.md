# Sunrise Node and Parachain Runtimes

This repository holds the codebase for Sunrise Standalone network daybreak as well as it's Rococo Parachain Dawn.


## Build & Run

Follow these steps to prepare a local Substrate development environment :hammer_and_wrench:

### Setup

If necessary, refer to the setup instructions at the
[Substrate Developer Hub](https://substrate.dev/docs/en/knowledgebase/getting-started/#manual-installation).

### Build

Once the development environment is set up, build the node template. This command will build the
[Wasm](https://substrate.dev/docs/en/knowledgebase/advanced/executor#wasm-execution) and
[native](https://substrate.dev/docs/en/knowledgebase/advanced/executor#native-execution) code:

```bash
cargo build --release
```
## Run

### Local Testnet

Sunrise Standalone
```
./target/release/sunrise purge-chain --dev
./target/release/sunrise --dev
```

Polkadot Local (rococo-v1 branch):
```
# Build up Polkadot relay chain specification
cds
cd polkadot
cargo build --release --features=real-overseer
./target/release/polkadot build-spec --chain rococo-local --raw --disable-default-bootnode > rococo_local.json

# Start up Polkadot relay chain
cds
cd polkadot
./target/release/polkadot purge-chain --chain ./rococo_local.json -d cumulus_relay1
./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay1 --validator --bob --port 50555

# Start the second relay chain node - in another session
cds
cd polkadot
./target/release/polkadot purge-chain --chain ./rococo_local.json -d cumulus_relay0
./target/release/polkadot --chain ./rococo_local.json -d cumulus_relay0 --validator --alice --port 50556

# Create the dawn paracahain
cdss
cargo build --release --bin dawn-collator
./target/release/dawn-collator export-genesis-state --parachain-id 888 > ./node/dawn/para-888-genesis
./target/release/dawn-collator export-genesis-wasm > ./node/dawn/para-888-wasm

# Start the collator node for the dawn parachain Alioth
rm -rf local-alioth/
./target/release/dawn-collator purge-chain --chain ./node/dawn/chain_spec/local.json -d local-alioth
./target/release/dawn-collator -d local-alioth --collator --name Alioth --ws-port 9946 --chain ./node/dawn/chain_spec/local.json --parachain-id 888 -- --chain ../polkadot/rococo_local.json

# Start the collator node for the dawn parachain Bibha
rm -rf local-bibha/
./target/release/dawn-collator purge-chain --chain ./node/dawn/chain_spec/local.json -d local-bibha
./target/release/dawn-collator -d local-bibha --collator --name Bibha --ws-port 9947 --chain ./node/dawn/chain_spec/local.json --parachain-id 888 -- --chain ../polkadot/rococo_local.json

```
