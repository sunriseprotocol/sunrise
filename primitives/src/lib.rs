#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{RuntimeDebug};
use sp_std::convert::{Into, TryFrom, TryInto};

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
	BTC = 3,

}

impl TryFrom<u8> for TokenSymbol {
	type Error = ();

	fn try_from(v: u8) -> Result<Self, Self::Error> {
		match v {
			0 => Ok(TokenSymbol::SRS),
			1 => Ok(TokenSymbol::DOT),
			2 => Ok(TokenSymbol::KSM),
			3 => Ok(TokenSymbol::BTC),
			_ => Err(()),
		}
	}
}

impl TryFrom<[u8; 32]> for CurrencyId {
	type Error = ();

	fn try_from(v: [u8; 32]) -> Result<Self, Self::Error> {
		if !v.starts_with(&[0u8; 29][..]) {
			return Err(());
		}

		// token
		if v[29] == 0 && v[31] == 0 {
			return v[30].try_into().map(CurrencyId::Native);
		}
		Err(())
	}
}

impl Into<[u8; 32]> for CurrencyId {
	fn into(self) -> [u8; 32] {
		let mut bytes = [0u8; 32];
		match self {
			CurrencyId::Native(token) => {
				bytes[30] = token as u8;
			}
			_ => {}
		}
		bytes
	}
}