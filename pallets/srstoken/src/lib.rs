#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, Parameter,  //debug,
};
use sp_runtime::{
	DispatchResult, RuntimeDebug,
	traits::{ CheckedSub, Saturating, Member, AtLeast32Bit, AtLeast32BitUnsigned, SaturatedConversion },
};
use frame_system::ensure_signed;
use sp_runtime::traits::One;
use codec::{Encode, Decode};
use frame_support::traits::Vec;
#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct TokenInfo<AccountId> {
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
	owner: AccountId,
}

impl<A> TokenInfo<A> {
	pub fn new(name_: Vec<u8>, symbol_: Vec<u8>, decimals_: u8 ,owner_: A) ->  TokenInfo<A> {
		TokenInfo {
			name: name_, 
			symbol: symbol_, 
			decimals: decimals_,
			owner: owner_,
		}
	}
}

pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Trait>::AccountId,
		<T as Trait>::Balance,
		<T as Trait>::AssetId,
	{
		Mint(AssetId, AccountId, Balance),
		Swap(AssetId, AccountId, AccountId, Balance),
		AssetId(),
		Burn(AssetId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		AmountZero,
		BalanceLow,
		BalanceZero,
		NotAllowed,
		AssetNotExists,
	}
}

type AccountIdOf<T> = <T as frame_system::Trait>::AccountId;
//type AssetIdOf<T> = <T as Trait>::AssetId;
decl_storage! {
	trait Store for Module<T: Trait> as SRSTokens {

		TokenInfos get(fn token_infos): map hasher(twox_64_concat) T::AssetId => Option<TokenInfo<AccountIdOf<T>>>;
		Balances get(fn balances):
			double_map hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => T::Balance;
		Allowances get(fn allowances):
			double_map  hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) (T::AccountId, T::AccountId) => T::Balance;
		NextAssetId get(fn next_asset_id): T::AssetId;
		TotalSupply get(fn total_supply): map hasher(twox_64_concat) T::AssetId => T::Balance;
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 100]
		fn asset_id_check(origin){
			let _origin = ensure_signed(origin)?;
			let _asset_id = <Self as Token<_, _>>::asset_id();
			Self::deposit_event(RawEvent::AssetId());
		}

		#[weight = 100]
		fn account_balance(origin,
			#[compact] asset_id: T::AssetId,
			account_id: T::AccountId,
		) {
			let _origin = ensure_signed(origin)?;
			let _bal = Self::balances(asset_id, account_id);
		}

		#[weight = 100]
		fn transfer(origin,
			#[compact] id: T::AssetId,
			target: T::AccountId,
			#[compact] amount: T::Balance
		) {
			let origin = ensure_signed(origin)?;
			<Self as Token<_, _>>::transfer(&id, &origin, &target, amount)?;
			Self::deposit_event(RawEvent::Swap(id, origin, target.clone(), amount));
		}

		#[weight = 100]
		fn create_new_asset(origin, name_: Vec<u8>, decimals_: u8, symbol_: Vec<u8>, owner_: T::AccountId, #[compact] initial_amount: T::Balance
		 ){
			let _origin = ensure_signed(origin)?;
			let token = TokenInfo::new(name_, symbol_, decimals_, owner_.clone());
			let asset_id = <Self as CreateTokenInfo<_, _>>::create_new_asset(token);
			<Self as CreateTokenInfo<_, _>>::mint(&asset_id, &owner_, initial_amount)?;
			Self::deposit_event(RawEvent::Mint(asset_id, owner_.clone(), initial_amount));
		 }
	}	
}

impl<T: Trait> Module<T> {

	pub fn impl_transfer(asset_id: &T::AssetId, from: &T::AccountId, to: &T::AccountId, value: T::Balance) -> DispatchResult {
		let _new_balance = Self::balances(asset_id, from)
			.checked_sub(&value)
			.ok_or(Error::<T>::BalanceLow)?;

		if from != to {
			<Balances<T>>::mutate(asset_id, from, |balance| *balance -= value);
			<Balances<T>>::mutate(asset_id, to, |balance| *balance += value);
		}

		Ok(())
	}
}

pub trait Token<AssetId, AccountId> {
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;
	//fn ownership(owner: &AccountId) -> Self::AccountId;
	//fn transferOwnership(new_owner: &AccountId) -> DispatchResult;
	//fn approve(asset_id: &AssetId, who: &AccountId, spender: &AccountId) -> Self::Balance 
	fn total_supply(asset_id: &AssetId) -> Self::Balance;
	fn balances(asset_id: &AssetId, who: &AccountId) -> Self::Balance;
	fn allowances(asset_id: &AssetId, owner: &AccountId, spender: &AccountId) -> Self::Balance;
	fn transfer(asset_id: &AssetId, from: &AccountId, to: &AccountId, value: Self::Balance) -> DispatchResult;
	fn transfer_from(asset_id: &AssetId, from: &AccountId, operator: &AccountId, to: &AccountId, value: Self::Balance) -> DispatchResult;
	fn asset_id() -> Self::AssetId;
	fn bal_conver(num: u128) -> Self::Balance;

}

pub trait CreateTokenInfo<AssetId, AccountId>: Token<AssetId, AccountId> {
	fn exists(asset_id: &AssetId) -> bool;
	fn create_new_asset(token_info: TokenInfo<AccountId>) -> AssetId;
	fn mint(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;
	fn burn(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;
	fn initial_amount(asset_id: &AssetId, account_id: &AccountId ) -> Self::Balance;
	fn bal_conv(num: u128) -> Self::Balance;
}


impl<T: Trait> Token<T::AssetId, T::AccountId> for Module<T> {
	type Balance = T::Balance;
	type AssetId = T::AssetId;

	fn bal_conver(num: u128) -> Self::Balance { 
		num.saturated_into::<Self::Balance>()
	}

	fn asset_id() -> Self::AssetId {
		Self::next_asset_id()
	}

	fn total_supply(asset_id: &T::AssetId) -> Self::Balance {
		Self::total_supply(&asset_id)
	}

	fn balances(asset_id: &T::AssetId, who: &T::AccountId) -> Self::Balance {
		Self::balances(asset_id, who)
	}

	fn allowances(asset_id: &T::AssetId, owner: &T::AccountId, spender: &T::AccountId) -> Self::Balance {
		Self::allowances(asset_id, (owner, spender))
	}

	fn transfer(asset_id: &T::AssetId, from: &T::AccountId, to: &T::AccountId, value: Self::Balance) -> DispatchResult {
		Self::impl_transfer(asset_id, from, to, value)
	}

	fn transfer_from(asset_id: &T::AssetId, from: &T::AccountId, operator: &T::AccountId, to: &T::AccountId, value: Self::Balance) -> DispatchResult {

		let new_allowance = Self::allowances(asset_id, (from, operator))
			.checked_sub(&value)
			.ok_or(Error::<T>::NotAllowed)?;

		if from != to {
			Self::impl_transfer(asset_id, from, to, value)?;
		}

		<Allowances<T>>::mutate(asset_id, (from, operator), |approved_balance| {
			*approved_balance = new_allowance;
		});
		Ok(())
	}
}

impl<T: Trait> CreateTokenInfo<T::AssetId, T::AccountId> for Module<T> {

	fn initial_amount(asset_id: &T::AssetId, account_id: &T::AccountId ) -> Self::Balance {
		Self::balances(&asset_id, account_id)
	}

	fn bal_conv(num: u128) -> Self::Balance { 
		num.saturated_into::<Self::Balance>()
	}
	

	fn exists(asset_id: &T::AssetId) -> bool {
		Self::token_infos(asset_id).is_some()
	}

	fn create_new_asset(token_info: TokenInfo< T::AccountId>) -> T::AssetId {
		let id = Self::next_asset_id();
		<NextAssetId<T>>::mutate(|id| *id += One::one());
		<TokenInfos<T>>::insert(id, &token_info);
		id
	}

	fn mint(asset_id: &T::AssetId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		ensure!(Self::exists(asset_id), Error::<T>::AssetNotExists);
		<Balances<T>>::mutate(asset_id, who, |balance| {
			*balance = balance.saturating_add(value);
		});
		<TotalSupply<T>>::mutate(asset_id, |supply| {
			*supply = supply.saturating_add(value);
		});

		Self::deposit_event(RawEvent::Mint(asset_id.clone(), who.clone(), value));
		Ok(())
	}

	fn burn(asset_id: &T::AssetId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		ensure!(Self::exists(asset_id), Error::<T>::AssetNotExists);
		let new_balance = Self::balances(asset_id, who)
			.checked_sub(&value)
			.ok_or(Error::<T>::BalanceLow)?;

		<Balances<T>>::mutate(asset_id, who, |balance| *balance = new_balance);
		<TotalSupply<T>>::mutate(asset_id, |supply| {
			*supply = supply.saturating_sub(value);
		});
		Self::deposit_event(RawEvent::Burn(asset_id.clone(), value));
		Ok(())
	} 
}