#![cfg_attr(not(feature = "std"), no_std)]

pub mod evm;
pub mod mocks;

use crate::evm::EvmAddress;

use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{RuntimeDebug};

use sp_std::{
	convert::{Into, TryFrom, TryInto},
	prelude::*,
};

pub type Decimals = u8;
pub type Balance = u128;
pub type AssetId = u64;
pub type AccountIndex = u32;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	SRS = 0,
	SUSD = 1,
	SBTC = 2,
}
impl TryFrom<u8> for TokenSymbol {
	type Error = ();

	fn try_from(v: u8) -> Result<Self, Self::Error> {
		match v {
			0 => Ok(TokenSymbol::SRS),
			1 => Ok(TokenSymbol::SUSD),
			2 => Ok(TokenSymbol::SBTC),
			_ => Err(()),
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DEXShare(TokenSymbol, TokenSymbol),
	ERC20(EvmAddress),
}

impl CurrencyId {
	pub fn is_token_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Token(_))
	}

	pub fn is_dex_share_currency_id(&self) -> bool {
		matches!(self, CurrencyId::DEXShare(_, _))
	}

	pub fn split_dex_share_currency_id(&self) -> Option<(Self, Self)> {
		match self {
			CurrencyId::DEXShare(token_symbol_0, token_symbol_1) => {
				Some((CurrencyId::Token(*token_symbol_0), CurrencyId::Token(*token_symbol_1)))
			}
			_ => None,
		}
	}

	pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		match (currency_id_0, currency_id_1) {
			(CurrencyId::Token(token_symbol_0), CurrencyId::Token(token_symbol_1)) => {
				Some(CurrencyId::DEXShare(token_symbol_0, token_symbol_1))
			}
			_ => None,
		}
	}
}

impl TryFrom<Vec<u8>> for CurrencyId {
	type Error = ();
	fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
		match v.as_slice() {
			b"SRS" => Ok(CurrencyId::Token(TokenSymbol::SRS)),
			b"SUSD" => Ok(CurrencyId::Token(TokenSymbol::SUSD)),
			b"SBTC" => Ok(CurrencyId::Token(TokenSymbol::SBTC)),
			_ => Err(()),
		}
	}
}