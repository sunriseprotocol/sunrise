//! A list of the different weight modules for our runtime.
#![allow(clippy::unnecessary_cast)]

//pub mod auction_manager;
//pub mod cdp_engine;
//pub mod cdp_treasury;
pub mod dex;
//pub mod emergency_shutdown;
pub mod evm;
pub mod evm_accounts;
//pub mod honzon;
pub mod incentives;
pub mod nft;
pub mod prices;
pub mod slip;
pub mod transaction_payment;

pub mod orml_auction;
pub mod orml_authority;
pub mod orml_gradually_update;
pub mod orml_oracle;
pub mod orml_rewards;
pub mod orml_tokens;
pub mod orml_vesting;