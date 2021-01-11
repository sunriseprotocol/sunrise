#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	Parameter, decl_module, decl_event, decl_storage, decl_error, ensure,
};

use sp_runtime::{
	RuntimeDebug, DispatchResult,
	traits::{
		CheckedSub, Saturating, Member, AtLeast32Bit, AtLeast32BitUnsigned
	},
};
use frame_system::ensure_signed;
use sp_runtime::traits::One;
use codec::{Encode, Decode};
//use sp_runtime::print;

type Symbol = u8;
type Name = u8;

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct TokenInfo<Name, Symbol, AccountId> {
	name: Name,
	symbol: Symbol,
	decimals: u8,
	owner: AccountId,
}

impl<A, B, C> TokenInfo<A, B, C> {
	fn new(name: A, symbol: B, decimals: u8 ,owner: C) -> Self {
		TokenInfo {
			symbol, 
			name, 
			decimals,
			owner,
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
		Issued(AssetId, AccountId, Balance),
		Transferred(AssetId, AccountId, AccountId, Balance),
		Destroyed(AssetId, AccountId, Balance),
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
		TokenInfos get(fn token_infos): map hasher(twox_64_concat) T::AssetId => Option<TokenInfo<Name, Symbol, AccountIdOf<T>>>;
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
		fn transfer(origin,
			#[compact] id: T::AssetId,
			target: T::AccountId,
			#[compact] amount: T::Balance
		) {
			let origin = ensure_signed(origin)?;

			<Self as Token<_, _>>::transfer(&id, &origin, &target, amount)?;
			Self::deposit_event(RawEvent::Transferred(id, origin, target.clone(), amount));
		}

		#[weight = 100]
		fn create_new_asset(origin, name: Name, decimals: u8, symbol: Symbol, owner: T::AccountId, #[compact] initial_amount: T::Balance ){

			let _origin = ensure_signed(origin)?;
			let cloned_owner = owner.clone();
			let second_owner = owner.clone();
			let asset_id  = <Self as CreateTokenInfo<_,_>>::create_new_asset(TokenInfo::new(name, symbol, decimals, owner));

			<Self as CreateTokenInfo<_,_>>::issue(&asset_id, &second_owner, initial_amount)?;
			Self::deposit_event(RawEvent::Issued(asset_id, cloned_owner, initial_amount));
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
	
	//fn ownership(owner: &AccountId) -> Self::AccountId;
	//fn transferOwnership(new_owner: &AccountId) -> DispatchResult;
	fn total_supply(asset_id: &AssetId) -> Self::Balance;
	fn balances(asset_id: &AssetId, who: &AccountId) -> Self::Balance;
	fn allowances(asset_id: &AssetId, owner: &AccountId, spender: &AccountId) -> Self::Balance;
	fn transfer(asset_id: &AssetId, from: &AccountId, to: &AccountId, value: Self::Balance) -> DispatchResult;
	fn transfer_from(asset_id: &AssetId, from: &AccountId, operator: &AccountId, to: &AccountId, value: Self::Balance) -> DispatchResult;
}

pub trait CreateTokenInfo<AssetId, AccountId>: Token<AssetId, AccountId> {
	fn exists(asset_id: &AssetId) -> bool;
	fn create_new_asset(token_info: TokenInfo<Name, Symbol, AccountId>) -> AssetId;
	fn issue(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;
	fn burn(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;
}


impl<T: Trait> Token<T::AssetId, T::AccountId> for Module<T> {
	type Balance = T::Balance;

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


	fn exists(asset_id: &T::AssetId) -> bool {
		Self::token_infos(asset_id).is_some()
	}

	fn create_new_asset(token_info: TokenInfo<Name, Symbol, T::AccountId>) -> T::AssetId {
		let id = Self::next_asset_id();
		<NextAssetId<T>>::mutate(|id| *id += One::one());
		<TokenInfos<T>>::insert(id, token_info);
		id
	}

	fn issue(asset_id: &T::AssetId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		ensure!(Self::exists(asset_id), Error::<T>::AssetNotExists);

		<Balances<T>>::mutate(asset_id, who, |balance| {
			*balance = balance.saturating_add(value);
		});
		<TotalSupply<T>>::mutate(asset_id, |supply| {
			*supply = supply.saturating_add(value);
		});

		Self::deposit_event(RawEvent::Issued(asset_id.clone(), who.clone(), value));
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
		Ok(())
	} 
}