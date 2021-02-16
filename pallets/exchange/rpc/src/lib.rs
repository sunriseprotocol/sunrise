use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
use std::sync::Arc;
use exchange_rpc_runtime_api::{BalanceInfo};

pub use self::gen_client::Client as ExchangeClient;

pub use exchange_rpc_runtime_api::ExchangeApi as ExchangeRuntimeApi;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BalanceRequest<Balance> {
	amount: Balance,
}

#[rpc]
pub trait ExchangeApi<BlockHash, AccountId, AssetId, Balance, PoolId,  ResponseType> {
	#[rpc(name = "price")]
	fn price(
		&self,
		pool_id: PoolId,
		balance_in: Balance,
		token_in:  Vec<u8>,
		balance_out: Balance,
		token_out:  Vec<u8>,
		at: Option<BlockHash>,
	) -> Result<ResponseType>;

	#[rpc(name = "calc_swap_exact_in")]
	fn calc_swap_exact_in(
		&self,
		pool_id: PoolId,
		balance_in: Balance,
		token_in: Vec<u8>,
		balance_out: Balance,
		token_out: Vec<u8>,
		token_amount_in: Balance,
		at: Option<BlockHash>,
	) -> Result<ResponseType>;

	#[rpc(name = "calc_swap_exact_out")]
	fn calc_swap_exact_out(
		&self,
		pool_id: PoolId,
		balance_in: Balance,
		token_in: Vec<u8>,
		balance_out: Balance,
		token_out: Vec<u8>,
		token_amount_out: Balance,
		at: Option<BlockHash>,
    ) -> Result<ResponseType>;
    
    #[rpc(name = "calc_join_pool_with_min_lptokens_given")]
	fn calc_join_pool_with_min_lptokens_given(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<BlockHash>,
    ) -> Result<ResponseType>;
    

	#[rpc(name = "calc_join_pool_with_max_collateral_taken")]
	fn calc_join_pool_with_max_collateral_taken(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<BlockHash>,
    ) -> Result<ResponseType>;
	
	#[rpc(name = "calc_exit_pool_with_min_collateral_received")]
	fn calc_exit_pool_with_min_collateral_received(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<BlockHash>,
    ) -> Result<ResponseType>;
	
	#[rpc(name = "calc_exit_pool_with_max_lp_given")]
	fn calc_exit_pool_with_max_lp_given(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<BlockHash>,
    ) -> Result<ResponseType>;
	
    
/*
  
	#[rpc(name = "pool_configuration")]
	fn pool_configuration(&self, pool_id: PoolId, at: Option<BlockHash>) -> Result<Vec<ResponseType>>;
*/
    
	#[rpc(name = "pool_reserves")]
	fn pool_reserves(&self, pool_address: AccountId, at: Option<BlockHash>) -> Result<Vec<ResponseType>>;
}

pub struct Exchange<A, B> {
	client: Arc<A>,
	_marker: std::marker::PhantomData<B>,
}

impl<A, B> Exchange<A, B> {
	pub fn new(client: Arc<A>) -> Self {
		Exchange {
			client,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

impl<A, Block, AccountId, AssetId, Balance, PoolId>
ExchangeApi<<Block as BlockT>::Hash, AccountId, AssetId, Balance, PoolId, BalanceInfo<AssetId, Balance>> for Exchange<A, Block>
where
	Block: BlockT,
	A: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	A::Api: ExchangeRuntimeApi<Block, AccountId, AssetId, Balance, PoolId>,
	AccountId: Codec,
	AssetId: Codec,
	Balance: Codec + MaybeDisplay + MaybeFromStr,
	PoolId: Codec,
{

	fn price(
		&self,
		pool_id: PoolId,
		balance_in: Balance,
		weight_in: Vec<u8>,
		balance_out: Balance,
		weight_out:  Vec<u8>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<AssetId, Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.price(&at, pool_id, balance_in, weight_in, balance_out, weight_out).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Price Calcutation".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calc_swap_exact_in(
		&self,
		pool_id: PoolId,
		balance_in: Balance,
		token_in: Vec<u8>,
		balance_out: Balance,
		token_out: Vec<u8>,
		token_amount_in: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<AssetId, Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.calc_swap_exact_in(&at, pool_id, balance_in, token_in, balance_out, token_out, token_amount_in).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Price Calcutation".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calc_swap_exact_out(
		&self,
		pool_id: PoolId,
		balance_in: Balance,
		token_in: Vec<u8>,
		balance_out: Balance,
		token_out: Vec<u8>,
		token_amount_out: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<AssetId, Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.calc_swap_exact_out(&at, pool_id, balance_in, token_in, balance_out, token_out, token_amount_out).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Price Calcutation".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calc_join_pool_with_min_lptokens_given(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<AssetId, Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.calc_join_pool_with_min_lptokens_given(&at, asset_a, asset_b, amount).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Price Calcutation".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calc_join_pool_with_max_collateral_taken(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<AssetId, Balance>> {
	let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.calc_join_pool_with_max_collateral_taken(&at, asset_a, asset_b, amount).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Price Calcutation".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calc_exit_pool_with_min_collateral_received(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<AssetId, Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.calc_exit_pool_with_min_collateral_received(&at, asset_a, asset_b, amount).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Price Calcutation".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn calc_exit_pool_with_max_lp_given(
		&self,
		asset_a: AssetId,
		asset_b: AssetId,
		amount: Balance,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<AssetId, Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.calc_exit_pool_with_max_lp_given(&at, asset_a, asset_b, amount).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Price Calcutation".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
/*
	fn pool_configuration(&self,
        pool_id: PoolId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<PoolConfig> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));

		api.pool_configuration(&at, pool_id).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Pool Configuration".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
*/
	fn pool_reserves(
		&self,
		pool_address: AccountId,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<Vec<BalanceInfo<AssetId, Balance>>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			self.client.info().best_hash));

		api.pool_reserves(&at, pool_address).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Invalid Pool Liquidity Retrieval".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
