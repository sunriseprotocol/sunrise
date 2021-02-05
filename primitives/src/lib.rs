#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{RuntimeDebug};

pub type Balance = u128;

pub type EvmAddress = sp_core::H160;


#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug , PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Native(TokenSymbol),
	ERC20(EvmAddress),
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	SRS = 0,
	DOT = 1,
	KSM = 2,
    ACA = 3,
    AUSD = 4,
}
