#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, Parameter
};
use frame_system::ensure_signed;
use sp_runtime::{
	DispatchResult, RuntimeDebug, 
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Bounded, One, CheckedAdd, Member},
};

//Debug string -> debug::info!("test value: {:?}", temp);			

use orml_traits::{MultiReservableCurrency, MultiCurrency};
//use orml_utilities::with_transaction_result;
use pallet_srstokens::{TokenInfo, Token};

pub trait Trait: frame_system::Trait + pallet_srstokens::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: MultiReservableCurrency<Self::AccountId>;
	type PoolId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type PoolConfigId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
	type SRSToken: Token<Self::AssetId, Self::AccountId>;
}

enum _CurveType {
    Stable,
	Oracle,
	Asset,
}

type PoolReserves = [u64; 4];
type CurrencyIds = [u64; 4];
type TokenWeights = [u64; 4];

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct LiquidityPool<Balance, PoolConfigId> {
	currency_ids: CurrencyIds,
	#[codec(compact)]
	base_amnt: Balance, 
	pool_config_id: PoolConfigId,
	pool_reserves: PoolReserves,
}

impl<A, B> LiquidityPool<A, B>{
	fn new(currency_ids: CurrencyIds, base_amnt: A, pool_config_id: B , pool_reserves: PoolReserves) -> 
	LiquidityPool<A, B> {
		LiquidityPool {
			currency_ids, 
			base_amnt,
			pool_config_id,
			pool_reserves
		}
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct PoolConfig<Balance, CurrencyIds, TokenWeights> {
	num_in_set: u32, 
	currency_ids: CurrencyIds,
	token_weights: TokenWeights,
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

impl<A, B, C> PoolConfig<A, B, C>{
	fn new(num_in_set: u32, currency_ids: B, token_weights: C ,fees: A, 
		depth: u32, slippage: A,alpha: A, kmpa: u32, curve_type: u8 ) ->  PoolConfig<A, B, C> {
		PoolConfig {
			num_in_set, 
			currency_ids,
			token_weights,
			fees, 
			depth,
			slippage,
			alpha,
			kmpa,
			curve_type,
		}
	}
}

type BalanceOf<T> = <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::Balance;
type LiquidityPool_<T> = LiquidityPool<BalanceOf<T>, <T as Trait>::PoolConfigId>;
type LiquidityPoolConfig_<T> = PoolConfig<BalanceOf<T>,CurrencyIds, TokenWeights>; 

decl_storage! {
	trait Store for Module<T: Trait> as pool {
		NextPoolId get(fn next_pool_id): T::PoolId;
		NextPoolConfigId get(fn next_pool_config_id): T::PoolConfigId;
		LiquidityPools get(fn pools): map hasher(twox_64_concat) T::PoolId => Option<LiquidityPool_<T>>;
		LiquidityPoolConfigs get(fn poolconfigs): map hasher(twox_64_concat) T::PoolConfigId => Option<LiquidityPoolConfig_<T>>;
	}
}

decl_event!{
	pub enum Event<T> where
        Balance = BalanceOf<T>,
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::PoolId,
	{
		CreateLiquidityPool(PoolId),
		AddLiquidity(AccountId, Balance, PoolId ),
		RemoveLiquidity(AccountId, Balance, PoolId),
		Swap(AccountId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		IdOverflow,
		InvalidId,
		InsufficientBalance,
		PastDeadline,
		PoolDoesntExist,
		ConfigDoesntExist,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 100]
		fn retrieve_config (origin, pool_identifier: T::PoolConfigId){
			ensure_signed(origin)?;
			ensure!(Self::poolconfigs(pool_identifier).is_some(), Error::<T>::ConfigDoesntExist);
		}

		#[weight = 100]
		fn retrieve_pool (origin, pool_identifier: T::PoolId){
			ensure_signed(origin)?;
			ensure!(Self::pools(pool_identifier).is_some(), Error::<T>::PoolDoesntExist);
		}

		#[weight = 100]
		fn liquidity_config_creation (origin, num: u32, currency_ids: CurrencyIds, token_weights: TokenWeights, depth: u32,
			fees: BalanceOf<T>, slippage: BalanceOf<T>, alpha: BalanceOf<T>, kmpa: u32, curve_type: u8)
		 {			
			ensure_signed(origin)?;
			let pool_config_id = Self::next_pool_config_id();
			let liq_config = PoolConfig::new(
				num, currency_ids, token_weights, fees, depth, slippage, alpha, kmpa, curve_type ); 

			<LiquidityPoolConfigs<T>>::insert(&pool_config_id, liq_config);
			NextPoolConfigId::<T>::try_mutate(|id| -> DispatchResult {
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::IdOverflow)?;
				Ok(())
			})?;
		}

		#[weight = 100]
		fn liquidity_pool_create (origin, base_amnt: BalanceOf<T> , currency_ids: CurrencyIds, pool_config_id: T::PoolConfigId, pool_reserves: PoolReserves, owner: T::AccountId ){
			ensure_signed(origin)?;
			let who = owner;
			let var: u8 = 4;
			let _token = TokenInfo::new(var, var, var, who);

			NextPoolId::<T>::try_mutate(|id| -> DispatchResult {
				let pool_id = *id;
				let liq_pool = LiquidityPool::new(
					currency_ids, base_amnt, pool_config_id, pool_reserves); 
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::IdOverflow)?;

				let _pool_config = Self::poolconfigs(&pool_config_id);
				LiquidityPools::<T>::insert(pool_id, liq_pool);

				Ok(())
			})?;
		}

		#[weight = 100]
		fn liquidity_add (origin, pool_id: T::PoolId, deadline: T::BlockNumber ){
			let _who = ensure_signed(origin)?;
			ensure!(deadline > <frame_system::Module<T>>::block_number(), Error::<T>::PastDeadline);
			let _pool = Self::pools(&pool_id).unwrap();

			LiquidityPools::<T>::try_mutate(pool_id, |_liquidity_pool| -> DispatchResult {
				Ok(())
			})?;
		}

		#[weight = 100]
		fn liquidity_remove (origin, _pool_id: T::PoolId ){
			let _who = ensure_signed(origin)?;
			LiquidityPools::<T>::try_mutate(_pool_id, |_liquidity_pool| -> DispatchResult {
				Ok(())
			})?;
		}

		#[weight = 100]
		fn liquidity_pool_remove (origin, _pool_id: T::PoolId, to:T::AccountId){
			let _who = ensure_signed(origin)?;
			NextPoolId::<T>::try_mutate(|_pool_id| -> DispatchResult {
				Ok(())
			})?;
		}

	}
}

impl<T: Trait> Module<T> {

	fn _price_calculation(_config: LiquidityPoolConfig_<T> , pool: LiquidityPool_<T>) -> u64 {
		let price_ratio = pool.pool_reserves[0] / pool.pool_reserves[1]; 
		price_ratio
	}

}

