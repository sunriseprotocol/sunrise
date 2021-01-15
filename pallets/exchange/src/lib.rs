#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, debug, Parameter, traits::BalanceStatus,
};
use frame_system::ensure_signed;
use sp_runtime::{
	DispatchResult, RuntimeDebug, 
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Bounded, One, CheckedAdd, Zero, Member},
};

use orml_traits::{MultiReservableCurrency, MultiCurrency};
use orml_utilities::with_transaction_result;
use pallet_srstokens::{Token};

pub trait Trait: frame_system::Trait + pallet_srstokens::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: MultiReservableCurrency<Self::AccountId>;
	type PoolId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type PoolConfigId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
	//type SRSToken: Token<T::AssetId, T::AccountId>;
}
enum CurveType {
    Stable,
	Oracle,
	Asset,
}

type PoolReserves = [u64; 4];
type CurrencyIds = [u64; 4];
type TokenWeights = [u64; 4];


#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct LiquidityPool<Balance> {
	currency_ids: CurrencyIds,
	#[codec(compact)]
	base_amnt: Balance, 
	pool_config_id: u32,
	pool_reserves: PoolReserves,
}

impl<A> LiquidityPool<A>{
	fn new(currency_ids: CurrencyIds, base_amnt: A, pool_config_id: u32 , pool_reserves: PoolReserves) -> 
	LiquidityPool<A> {
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
type CurrencyIdOf<T> = <<T as Trait>::Currency as MultiCurrency<<T as frame_system::Trait>::AccountId>>::CurrencyId;
type LiquidityPool_<T> = LiquidityPool<BalanceOf<T>>;
type LiquidityPoolConfig_<T> = PoolConfig<BalanceOf<T>,CurrencyIds, TokenWeights>; 

decl_storage! {
	trait Store for Module<T: Trait> as pool {
		NextPoolId get(fn next_pool_id): T::PoolId;
		NextPoolConfigId get(fn next_pool_config_id): T::PoolConfigId;
		LiquidityPools get(fn pools): map hasher(twox_64_concat) T::PoolId => Option<LiquidityPool_<T>>;
		LiquidityPoolConfigs get(fn poolconfigs): map hasher(twox_64_concat) T::PoolConfigId => Option<LiquidityPoolConfig_<T>>;
	}

}

decl_event!(
	pub enum Event<T> where
	//	LiquidityPool = LiquidityPool_<T>,
        Balance = BalanceOf<T>,
		<T as frame_system::Trait>::AccountId,
		Pool_Config = LiquidityPoolConfig_<T>
 	//	Currency = CurrencyIdOf<T>,
	{
	//	AddLiquidity(PoolId, Balance, Balance),
	//	CreatePool(),
	//	RemoveLiquidity(),
	//	Swap()
	    CreatePoolConfig(Pool_Config),
		Swap(AccountId, Balance),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		IdOverflow,
		InvalidId,
		InsufficientBalance,
		NotOwner,
		TooLate,
		PoolDoesntExist
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 1000]
		fn retrieve_config (origin, pool_identifier: T::PoolConfigId){
			let who = ensure_signed(origin)?;
			
			let pool_config = Self::poolconfigs(pool_identifier);
			debug::info!("BALANCE sent by: {:?}", pool_config);			

		}



		#[weight = 1000]
		fn liquidity_config_creation (origin, num: u32, currency_ids: CurrencyIds, token_weights: TokenWeights, depth: u32,
			fees: BalanceOf<T>, slippage: BalanceOf<T>, alpha: BalanceOf<T>, kmpa: u32, curve_type: u8)
		 {			
			let who = ensure_signed(origin)?;
			let pool_config_id = Self::next_pool_config_id();
			let liq_config = PoolConfig::new(
				num, currency_ids, token_weights, fees, depth, slippage, alpha, kmpa, curve_type ); 
				
			debug::info!("config_id sent by: {:?}", pool_config_id);			
			debug::info!("data sent by: {:?}", liq_config);			

			<LiquidityPoolConfigs<T>>::insert(pool_config_id, &liq_config);
			let temp = Self::poolconfigs(pool_config_id);
			debug::info!("test sent by: {:?}", temp);			
		//	Self::deposit_event(RawEvent::CreatePoolConfig(&liq_config));

			NextPoolConfigId::<T>::try_mutate(|id| -> DispatchResult {
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::IdOverflow)?;
				Ok(())

			})?;

		}

		#[weight = 1000]
		fn liquidity_pool_create (origin, base_amnt: BalanceOf<T> , currency_ids: CurrencyIds, pool_config_id: u32, pool_reserves: PoolReserves ){
			let _who = ensure_signed(origin)?;

			NextPoolId::<T>::try_mutate(|id| -> DispatchResult {
				let pool_id = *id;
				let liq_pool = LiquidityPool::new(
					currency_ids, base_amnt, pool_config_id, pool_reserves); 

				*id = id.checked_add(&One::one()).ok_or(Error::<T>::IdOverflow)?;
				LiquidityPools::<T>::insert(pool_id, liq_pool);
				Ok(())
			})?;
		}

		#[weight = 1000]
		fn liquidity_add (origin, pool_id: T::PoolId, deadline: T::BlockNumber ){
			let _who = ensure_signed(origin)?;
			ensure!(deadline > <frame_system::Module<T>>::block_number(), Error::<T>::TooLate);
			let mut pool = Self::pools(&pool_id).unwrap();

			LiquidityPools::<T>::try_mutate(pool_id, |_liquidity_pool| -> DispatchResult {
				Ok(())
			})?;
		}

		#[weight = 1000]
		fn liquidity_remove (origin, _pool_id: T::PoolId ){
			let _who = ensure_signed(origin)?;
			LiquidityPools::<T>::try_mutate(_pool_id, |_liquidity_pool| -> DispatchResult {
				Ok(())
			})?;
		}

		#[weight = 1000]
		fn liquidity_pool_remove (origin, _pool_id: T::PoolId, to:T::AccountId){
			let _who = ensure_signed(origin)?;
			NextPoolId::<T>::try_mutate(|_pool_id| -> DispatchResult {
				Ok(())
			})?;
		}
/*
		#[weight = 1000]
		fn swap(origin, pool_id: T::PoolId, amount: BalanceOf<T> ){
			let _who = ensure_signed(origin)?;

			let LiquidityPool lp = pools(pool_id);
			let LiquidityPoolConfig pconfig = poolconfigs(lp.pool_config_id);

			Token<_,_>::transfer(&origin,
				Token<>::Asset_id(pconfig.currency_ids[0]),
				lp.pool_reserves[0].AccountId,
				amount);
			
			Token<_,_>::transfer(&origin,
				Token<_,_>::Asset_id(pconfig.currency_ids[1]),
				lp.pool_reserves[1].AccountId,
				amount);


			lp.pool_reserves[0] = lp.pool_reserves[0] + amount * pconfig.token_weights[0] ; //+ slip
			lp.pool_reserves[1] = lp.pool_reserves[1] - amount * pconfig.token_weights[1] ; //+ slip
			_sync()
			


			LiquidityPools::<T>::try_mutate(pool_id, |_liquidity_pool| -> DispatchResult {

					Ok(())
				})?;
		} */
	}
}

impl<T: Trait> Module<T> {

	fn price_calculation(config: LiquidityPoolConfig_<T> , pool: LiquidityPool_<T>) -> u64 {
		let price_ratio = pool.pool_reserves[0] / pool.pool_reserves[1]; 
		price_ratio
	}

}

