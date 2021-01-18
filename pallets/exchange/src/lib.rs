#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, Parameter
};
use sp_runtime::{
	DispatchResult, RuntimeDebug, ModuleId,
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Bounded, Member,  AccountIdConversion},
};
use frame_system::ensure_signed;
use codec::{Encode, Decode};
use frame_support::traits::{Get, Vec};
#[macro_use]
extern crate alloc;
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
	type ModuleId: Get<ModuleId>;

}

enum _CurveType {
    Stable,
	Oracle,
	Asset,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct LiquidityPool<Balance, PoolConfigId, AssetId> {
	currency_ids: Vec<AssetId>,
	#[codec(compact)]
	base_amnt: Balance, 
	pool_config_id: PoolConfigId,
	pool_reserves: Vec<Balance>,
}

impl<A, B, C> LiquidityPool<A, B, C>{
	fn new(currency_ids: Vec<C>, base_amnt: A, pool_config_id: B , pool_reserves: Vec<A>) -> 
	LiquidityPool<A, B, C> {
		LiquidityPool {
			currency_ids, 
			base_amnt,
			pool_config_id,
			pool_reserves
		}
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct PoolConfig<Balance, AssetId> {
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

impl<A, B> PoolConfig<A, B>{
	fn new(num_in_set: u32, currency_ids: Vec<B>, token_weights: Vec<u64> ,fees: A, 
		depth: u32, slippage: A,alpha: A, kmpa: u32, curve_type: u8 ) ->  PoolConfig<A, B> {
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
type AssetIdOf<T> = <T as pallet_srstokens::Trait>::AssetId;

type LiquidityPool_<T> = LiquidityPool<BalanceOf<T>, <T as Trait>::PoolConfigId, AssetIdOf<T> >;
type LiquidityPoolConfig_<T> = PoolConfig<BalanceOf<T>, AssetIdOf<T> >; 

decl_storage! {
	trait Store for Module<T: Trait> as pool {
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
		BadNumGen,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 100]
		fn liquidity_config_creation (origin, id: u32, currency_ids: Vec<AssetIdOf<T>>, token_weights: Vec<u64>, depth: u32,
			fees: BalanceOf<T>, slippage: BalanceOf<T>, alpha: BalanceOf<T>, kmpa: u32, curve_type: u8)
		 {			
			ensure_signed(origin)?;
			let pool_config_id = Self::pool_config_id(id);		
			ensure!(Self::poolconfigs(pool_config_id).is_some(), Error::<T>::BadNumGen);

			let liq_config = PoolConfig::new(
				id, currency_ids, token_weights, fees, depth, slippage, alpha, kmpa, curve_type ); 

			<LiquidityPoolConfigs<T>>::insert(&pool_config_id, liq_config);
		
		}

		#[weight = 100]
		fn liquidity_pool_create (origin, id: u32, base_amnt: BalanceOf<T> , currency_ids: Vec<AssetIdOf<T>>, pool_config_id: T::PoolConfigId, pool_reserves: Vec<BalanceOf<T>>, owner: T::AccountId ){
			ensure_signed(origin)?;
			let who = owner;

			let numb: u8 = 4;
			let temp: Vec<u8> = vec![numb];
			let _token = TokenInfo::new(temp.clone(), temp, numb, who);
			let liq_pool = LiquidityPool::new(
				currency_ids, base_amnt, pool_config_id, pool_reserves); 

			let pool_id = Self::pool_id(id);
			ensure!(!Self::pools(pool_id).is_some(), Error::<T>::BadNumGen);

			let _pool_config = &pool_config_id;
			ensure!(Self::poolconfigs(&pool_config_id).is_some(), Error::<T>::ConfigDoesntExist);

			LiquidityPools::<T>::insert(pool_id, liq_pool);

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
		}

	}
}

impl<T: Trait> Module<T> {

//	fn account_id() -> T::AccountId  {
//		T::ModuleId::get().into_account()
//	}

	fn pool_id(num: u32) -> T::PoolId {
		num.into()
	}

	fn pool_config_id(num: u32) -> T::PoolConfigId {
		num.into()
	}

}

