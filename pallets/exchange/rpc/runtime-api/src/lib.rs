#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_runtime::traits::{MaybeDisplay, MaybeFromStr};
use sp_std::prelude::*;


#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct BalanceInfo<AssetId, Balance> {
	#[cfg_attr(feature = "std", serde(bound(serialize = "Balance: std::fmt::Display")))]
	#[cfg_attr(feature = "std", serde(serialize_with = "serialize_as_string"))]
	#[cfg_attr(feature = "std", serde(bound(deserialize = "Balance: std::str::FromStr")))]
	#[cfg_attr(feature = "std", serde(deserialize_with = "deserialize_from_string"))]
	pub amount: Balance,
	pub asset_id: Option<AssetId>,
}

/*
#[derive(Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct PoolConfigInfo<Balance, AssetId> {
	#[cfg_attr(feature = "std", serde(bound(serialize = "Balance: std::fmt::Display")))]
	#[cfg_attr(feature = "std", serde(serialize_with = "serialize_as_string"))]
	#[cfg_attr(feature = "std", serde(bound(deserialize = "Balance: std::str::FromStr")))]
	#[cfg_attr(feature = "std", serde(deserialize_with = "deserialize_from_string"))]
	num_in_set: u32, 
	currency_ids: Vec<AssetId>,
	token_weights: Vec<u64>,
	#[codec(compact)]
	fees: Balance, 
	depth: u32,
	#[codec(compact)]
	slippage: Balance,
	#[codec(compact)]
	alpha: Balance,
	kmpa: u32,
	curve_type: u8
}
*/
#[cfg(feature = "std")]
fn serialize_as_string<S: Serializer, T: std::fmt::Display>(t: &T, serializer: S) -> Result<S::Ok, S::Error> {
	serializer.serialize_str(&t.to_string())
}

#[cfg(feature = "std")]
fn deserialize_from_string<'de, D: Deserializer<'de>, T: std::str::FromStr>(deserializer: D) -> Result<T, D::Error> {
	let s = String::deserialize(deserializer)?;
	s.parse::<T>()
		.map_err(|_| serde::de::Error::custom("Parse from string failed"))
}

sp_api::decl_runtime_apis! {
	pub trait ExchangeApi<AccountId, AssetId, Balance, PoolId> where
		AccountId: Codec,
		AssetId: Codec,
		Balance: Codec + MaybeDisplay + MaybeFromStr,
		PoolId: Codec,
	{
		fn price(
			pool_id: PoolId,
			balance_in: Balance,
			token_in: Vec<u8>,
			balance_out: Balance,
			token_out: Vec<u8>,
		) -> BalanceInfo<AssetId, Balance>;

	/*	fn pool_configuration(
			pool_id: PoolId,
		) -> PoolConfigInfo<Balance, AssetId>;
*/
		fn pool_reserves(
			pool_address: AccountId,
		) -> Vec<BalanceInfo<AssetId, Balance>>;

		fn calc_swap_exact_in(
			pool_id: PoolId,
			balance_in: Balance,
			token_in: Vec<u8>,
			balance_out: Balance,
			token_out: Vec<u8>,
			token_amount_in: Balance,
		) -> BalanceInfo<AssetId, Balance>;
	
		fn calc_swap_exact_out(
			pool_id: PoolId,
			balance_in: Balance,
			token_in: Vec<u8>,
			balance_out: Balance,
			token_out: Vec<u8>,
			token_amount_out: Balance,
		) -> BalanceInfo<AssetId, Balance>;
	
		fn calc_join_pool_with_min_lptokens_given(
			asset_a: AssetId,
			asset_b: AssetId,
			amount: Balance,
		) -> BalanceInfo<AssetId, Balance>;

		fn calc_join_pool_with_max_collateral_taken(
			asset_a: AssetId,
			asset_b: AssetId,
			amount: Balance,
		) -> BalanceInfo<AssetId, Balance>;

		fn calc_exit_pool_with_min_collateral_received(
			asset_a: AssetId,
			asset_b: AssetId,
			amount: Balance,
		) -> BalanceInfo<AssetId, Balance>;
	
		fn calc_exit_pool_with_max_lp_given(
			asset_a: AssetId,
			asset_b: AssetId,
			amount: Balance,
		) -> BalanceInfo<AssetId, Balance>;

	}
}


