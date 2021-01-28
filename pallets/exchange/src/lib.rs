#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, Parameter
};
use sp_runtime::{
	DispatchResult, RuntimeDebug, ModuleId,
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize, Bounded, Member,  AccountIdConversion, SaturatedConversion},
};
use frame_system::ensure_signed;
use codec::{Encode, Decode};
use frame_support::traits::{Get, Vec};
#[macro_use]
extern crate alloc;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

//Debug string -> debug::info!("test value: {:?}", temp);			
use orml_traits::{MultiReservableCurrency, MultiCurrency};
//use orml_utilities::with_transaction_result;
use pallet_srstokens::{TokenInfo, Token, CreateTokenInfo};

	pub trait Trait: frame_system::Trait + pallet_srstokens::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Currency: MultiReservableCurrency<Self::AccountId>;
	type PoolId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type PoolConfigId: Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize + Bounded;
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
	type Token: Token<Self::AssetId, Self::AccountId>;
	type ModuleId: Get<ModuleId>;
	type TokenFunctions: CreateTokenInfo<Self::AssetId, Self::AccountId>;

}

enum _CurveType {
    Stable,
	Oracle,
	Asset,
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct LiquidityPool<A, B, C> {
	currency_ids: Vec<C>,
	#[codec(compact)]
	lp_token_id: C, 
	pool_config_id: B,
	pool_reserves: Vec<A>,
}

impl<A, B, C> LiquidityPool<A, B, C>{
	fn new(currency_ids: Vec<C>, lp_token_id: C, pool_config_id: B , pool_reserves: Vec<A>) -> 
	LiquidityPool<A, B, C> {
		LiquidityPool {
			currency_ids, 
			lp_token_id,
			pool_config_id,
			pool_reserves
		}
	}
}

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct PoolConfig<A, B> {
	num_in_set: u32, 
	currency_ids: Vec<B>,
	token_weights: Vec<u64>,
	#[codec(compact)]
	fees: A, 
	depth: u32,
	#[codec(compact)]
	slippage: A,
	#[codec(compact)]
	alpha: A,
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
type TokenBalanceOf<T> = <T as pallet_srstokens::Trait>::Balance;

type LiquidityPool_<T> = LiquidityPool<BalanceOf<T>, <T as Trait>::PoolConfigId, AssetIdOf<T> >;
type LiquidityPoolConfig_<T> = PoolConfig<BalanceOf<T>, AssetIdOf<T> >; 

decl_storage! {
	trait Store for Module<T: Trait> as pool {
		LiquidityPools get(fn pools): map hasher(twox_64_concat) T::PoolId => Option<LiquidityPool_<T>>;
		//tuple poolconfigs into pool
		LiquidityPoolConfigs get(fn poolconfigs): map hasher(twox_64_concat) T::PoolConfigId => Option<LiquidityPoolConfig_<T>>;
	}
}

decl_event!{
	pub enum Event<T> where
		TokenBalance = TokenBalanceOf<T>,
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::PoolId,
		Pair = ( AssetIdOf<T>,TokenBalanceOf<T>),
	{
		CreateLiquidityPool(PoolId),
		AddLiquidity(AccountId, TokenBalance, PoolId),
		RemoveLiquidity(AccountId, TokenBalance, PoolId),
		Swap(AccountId, Option<Pair>, Option<Pair>, PoolId),
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
		PoolSizeError,
		FailedTransfer,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 100]
		fn liquidity_config_creation(origin, id: u32, currency_ids: Vec<AssetIdOf<T>>, token_weights: Vec<u64>, depth: u32,
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
		fn liquidity_pool_create(origin, id: u32 , currency_ids: Vec<AssetIdOf<T>>, pool_config_id: T::PoolConfigId, pool_reserves: Vec<BalanceOf<T>>, owner: T::AccountId, asset_id: AssetIdOf<T> ){
			ensure_signed(origin)?;
			let who = owner;

			let numb: u8 = 4;
			let temp: Vec<u8> = vec![numb];

			let pool_id = Self::pool_id(id);
			ensure!(!Self::pools(pool_id).is_some(), Error::<T>::BadNumGen);

			let _pool_config = &pool_config_id;
			ensure!(Self::poolconfigs(&pool_config_id).is_some(), Error::<T>::ConfigDoesntExist);

			let lp_token = TokenInfo::new(temp.clone(), temp, numb, who);
			//check asset_id / fix to be auto gen'd
			let asset_id = T::TokenFunctions::create_new_asset(lp_token, asset_id);

			let liq_pool = LiquidityPool::new(
				currency_ids, asset_id, pool_config_id, pool_reserves); 

			let initial_bal = T::TokenFunctions::initial_amount(&asset_id, &Self::account_id());			
			T::TokenFunctions::mint(&asset_id, &Self::account_id(), initial_bal)?;

			LiquidityPools::<T>::insert(pool_id, liq_pool);

		}

		#[weight = 100]
		fn liquidity_add(origin, pool_id: T::PoolId, deadline: T::BlockNumber, currencies: Vec<AssetIdOf<T>>, balances: Vec<TokenBalanceOf<T>> , temp_bal: TokenBalanceOf<T>){
			
			let who = ensure_signed(origin)?;
			ensure!(deadline > <frame_system::Module<T>>::block_number(), Error::<T>::PastDeadline);
			ensure!(Self::pools(pool_id).is_some(), Error::<T>::PoolDoesntExist);
			ensure!(currencies.len() == balances.len(),  Error::<T>::PoolSizeError);

			Self::add_liquidity(&who, currencies, balances, pool_id, temp_bal)?;
		}

		#[weight = 100]
		fn liquidity_remove(origin, pool_id: T::PoolId, deadline: T::BlockNumber, currencies: Vec<AssetIdOf<T>>, balances: Vec<TokenBalanceOf<T>> , lp_amount: TokenBalanceOf<T>){

			let who = ensure_signed(origin)?;
			ensure!(deadline > <frame_system::Module<T>>::block_number(), Error::<T>::PastDeadline);
			ensure!(Self::pools(pool_id).is_some(), Error::<T>::PoolDoesntExist);
			ensure!(currencies.len() == balances.len(),  Error::<T>::PoolSizeError);

			Self::remove_liquidity(&who, currencies, balances, pool_id, lp_amount)?;
		}
		
		#[weight = 100]
		fn exchange(origin, pool_id: T::PoolId, deadline: T::BlockNumber,  currencies_in: Vec<AssetIdOf<T>>, balances_in: Vec<TokenBalanceOf<T>>,
			currencies_out: Vec<AssetIdOf<T>>, balances_out: Vec<TokenBalanceOf<T>>){
				let who = ensure_signed(origin)?;
				ensure!(deadline > <frame_system::Module<T>>::block_number(), Error::<T>::PastDeadline);
				//ensure weights match
				//ensure!(Self::weight_watcher(pool_id, ), Error::<T>::WeightMismatch);
				Self::swap(&who, pool_id, currencies_in, balances_in, currencies_out, balances_out)?;
				
		}
	}
}

impl<T: Trait> Module<T> {

	pub fn fixed_bal(input: TokenBalanceOf<T>) ->  BalanceOf<T> {
		let temp = input.saturated_into::<u128>();
		temp.saturated_into::<BalanceOf<T>>()
	}

	pub fn fixed_token_bal(input: TokenBalanceOf<T>) -> u128 {
		input.saturated_into::<u128>()
	}

	fn add_liquidity(who: &T::AccountId, currencies: Vec<AssetIdOf<T>>, balances: Vec<TokenBalanceOf<T>>, pool_id: T::PoolId, temp_bal: TokenBalanceOf<T>) -> DispatchResult {
		
		// need to tuple currencyid and reserve @_@
		let mut mutant_pool = Self::pools(&pool_id).unwrap();
		
		let mut lp_total = temp_bal;
		for (x, _val) in currencies.iter().enumerate() {
			T::Token::transfer(&currencies[x], who, &Self::account_id(), T::Token::bal_conver(Self::fixed_token_bal(balances[x])))?;
			lp_total = lp_total + balances[x];
			mutant_pool.pool_reserves[x] += Self::fixed_bal(balances[x]);
		}
		let lp_token = mutant_pool.lp_token_id;
		<LiquidityPools<T>>::mutate(&pool_id, |pool| *pool = Some(mutant_pool));
		T::TokenFunctions::mint(&lp_token, &who, T::TokenFunctions::bal_conv(Self::fixed_token_bal(lp_total)))?;

		Self::deposit_event(RawEvent::AddLiquidity(who.clone(), temp_bal, pool_id));
		Ok(())
	}

	fn remove_liquidity(who: &T::AccountId, currencies: Vec<AssetIdOf<T>>, balances: Vec<TokenBalanceOf<T>>, pool_id: T::PoolId, lp_amount: TokenBalanceOf<T>) -> DispatchResult {
		
		let mut mutant_pool = Self::pools(&pool_id).unwrap();
		
		for (x, _val) in currencies.iter().enumerate() {
			T::Token::transfer(&currencies[x], &Self::account_id(), who, T::Token::bal_conver(Self::fixed_token_bal(balances[x])))?;
			mutant_pool.pool_reserves[x] -= Self::fixed_bal(balances[x]);
		}
		let lp_token = mutant_pool.lp_token_id;
		<LiquidityPools<T>>::mutate(&pool_id, |pool| *pool = Some(mutant_pool));
		T::TokenFunctions::burn(&lp_token, &who, T::TokenFunctions::bal_conv(Self::fixed_token_bal(lp_amount)))?;

		Self::deposit_event(RawEvent::RemoveLiquidity(who.clone(), lp_amount, pool_id));
		Ok(())
	}

	fn swap(who: &T::AccountId, pool_id: T::PoolId, currencies_in: Vec<AssetIdOf<T>>, balances_in: Vec<TokenBalanceOf<T>>,
		currencies_out: Vec<AssetIdOf<T>>, balances_out: Vec<TokenBalanceOf<T>>) -> DispatchResult {
			let mut _in_pair;
			let mut _out_pair;
			let mut mutant_pool = Self::pools(&pool_id).unwrap();
			//received by lp
			for (x, _val) in currencies_in.iter().enumerate() {
				_in_pair = Some((currencies_in[x], balances_in[x]));
				T::Token::transfer(&currencies_in[x], who, &Self::account_id(), T::Token::bal_conver(Self::fixed_token_bal(balances_in[x])))?;
				mutant_pool.pool_reserves[x] += Self::fixed_bal(balances_in[x]);
				
			}
			//received by user
			for (x, _val) in currencies_out.iter().enumerate() {
				_out_pair = Some((currencies_out[x], balances_out[x]));
				T::Token::transfer(&currencies_out[x], &Self::account_id(), who, T::Token::bal_conver(Self::fixed_token_bal(balances_out[x])))?;
				mutant_pool.pool_reserves[x] -= Self::fixed_bal(balances_out[x]);
			}

			<LiquidityPools<T>>::mutate(&pool_id, |pool| *pool = Some(mutant_pool));
	
		//	Self::deposit_event(RawEvent::Swap(  who.clone(), Some(in_pair) , Some(out_pair) ,pool_id )   );
			Ok(())
		
	}

	fn account_id() -> T::AccountId  {
		T::ModuleId::get().into_account()
	}

	fn pool_id(num: u32) -> T::PoolId {
		num.into()
	}

	fn pool_config_id(num: u32) -> T::PoolConfigId {
		num.into()
	}


}

