//! A list of the different weight modules for our runtime.
#![allow(clippy::unnecessary_cast)]

pub mod srs_pallet_auction_manager;
pub mod srs_pallet_currencies;
pub mod srs_pallet_dex;
pub mod srs_pallet_emergency_shutdown;
pub mod srs_pallet_evm;
pub mod srs_pallet_evm_accounts;
pub mod srs_pallet_incentives;
pub mod srs_pallet_nft;
pub mod srs_pallet_prices;
pub mod srs_pallet_slip;
pub mod srs_pallet_shy;
pub mod srs_pallet_shy_engine;
pub mod srs_pallet_shy_treasury;
pub mod srs_pallet_transaction_payment;

pub mod orml_auction;
pub mod orml_authority;
pub mod orml_gradually_update;
pub mod orml_oracle;
pub mod orml_rewards;
pub mod orml_tokens;
pub mod orml_vesting;
