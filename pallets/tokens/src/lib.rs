#![cfg_attr(not(feature = "std"), no_std)]
use frame_support::{
	decl_event, decl_module, decl_storage, decl_error, ensure, Parameter,  //debug,
};
use sp_runtime::{
	DispatchResult, RuntimeDebug,
	traits::{ CheckedSub, Saturating, Member, AtLeast32Bit, AtLeast32BitUnsigned, SaturatedConversion },
};
use frame_system::ensure_signed;
use codec::{Encode, Decode};
use frame_support::traits::Vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Clone, RuntimeDebug, Eq, PartialEq)]
pub struct TokenInfo<A> {
	name: Vec<u8>,
	symbol: Vec<u8>,
	decimals: u8,
	owner: A,
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

pub trait Config: frame_system::Config {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;
	type AssetId: Parameter + AtLeast32Bit + Default + Copy;
}

decl_event! {
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		<T as Config>::Balance,
		<T as Config>::AssetId,
	{
		Mint(AssetId, AccountId, Balance),
		Swap(AssetId, AccountId, AccountId, Balance),
		AssetId(),
		Burn(AssetId, Balance),
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		InsufficientBalance,
		InsufficientAllowance,
		InvalidAsset,
		InvalidIdGeneration,
	}
}

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
decl_storage! {
	trait Store for Module<T: Config> as SRSTokens {

		TokenInfos get(fn token_infos): map hasher(twox_64_concat) T::AssetId => Option<TokenInfo<AccountIdOf<T>>>;
		Balances get(fn balances):
			double_map hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => T::Balance;
		Allowances get(fn allowances):
			double_map  hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) (T::AccountId, T::AccountId) => T::Balance;
		TotalSupply get(fn total_supply): map hasher(twox_64_concat) T::AssetId => T::Balance;
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		type Error = Error<T>;
		fn deposit_event() = default;

		#[weight = 100]
		fn transfer(origin,
			#[compact] asset_id: T::AssetId,
			to: T::AccountId,
			#[compact] amount: T::Balance
		) {
			let origin = ensure_signed(origin)?;
			ensure!(Self::token_infos(asset_id).is_some(), Error::<T>::InvalidAsset);
			<Self as Token<_, _>>::transfer(&asset_id, &origin, &to, amount)?;
			Self::deposit_event(RawEvent::Swap(asset_id, origin, to.clone(), amount));
		}

		#[weight = 100]
		fn create_new_asset(origin, name_: Vec<u8>, decimals_: u8, symbol_: Vec<u8>, owner_: T::AccountId, #[compact] initial_amount: T::Balance, asset_id_: T::AssetId
		 ){
			let _origin = ensure_signed(origin)?;
			let token = TokenInfo::new(name_, symbol_, decimals_, owner_.clone());
			ensure!(!Self::token_infos(asset_id_).is_some(), Error::<T>::InvalidIdGeneration);
			let asset_id = <Self as CreateTokenInfo<_, _>>::create_new_asset(token, asset_id_);
			<Self as CreateTokenInfo<_, _>>::mint(&asset_id, &owner_, initial_amount)?;
			Self::deposit_event(RawEvent::Mint(asset_id, owner_.clone(), initial_amount));
		 }
	}	
}

impl<T: Config> Module<T> {

	pub fn impl_transfer(asset_id: &T::AssetId, from: &T::AccountId, to: &T::AccountId, value: T::Balance) -> DispatchResult {
		let _new_balance = Self::balances(asset_id, from)
			.checked_sub(&value)
			.ok_or(Error::<T>::InsufficientBalance)?;

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
	fn bal_conver(num: u128) -> Self::Balance;

}

pub trait CreateTokenInfo<AssetId, AccountId>: Token<AssetId, AccountId> {
	fn exists(asset_id: &AssetId) -> bool;
	fn create_new_asset(token_info: TokenInfo<AccountId>, asset_id: AssetId) -> AssetId;
	fn mint(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;
	fn burn(asset_id: &AssetId, who: &AccountId, value: Self::Balance) -> DispatchResult;
	fn initial_amount(asset_id: &AssetId, account_id: &AccountId ) -> Self::Balance;
	fn bal_conv(num: u128) -> Self::Balance;
}


impl<T: Config> Token<T::AssetId, T::AccountId> for Module<T> {
	type Balance = T::Balance;
	type AssetId = T::AssetId;

	fn bal_conver(num: u128) -> Self::Balance { 
		num.saturated_into::<Self::Balance>()
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

	fn transfer(asset_id: &T::AssetId, from: &T::AccountId, to: &T::AccountId, value: T::Balance) -> DispatchResult {
		Self::impl_transfer(asset_id, from, to, value)
	}

	fn transfer_from(asset_id: &T::AssetId, from: &T::AccountId, operator: &T::AccountId, to: &T::AccountId, value: T::Balance) -> DispatchResult {

		let new_allowance = Self::allowances(asset_id, (from, operator))
			.checked_sub(&value)
			.ok_or(Error::<T>::InsufficientAllowance)?;

		if from != to {
			Self::impl_transfer(asset_id, from, to, value)?;
		}

		<Allowances<T>>::mutate(asset_id, (from, operator), |approved_balance| {
			*approved_balance = new_allowance;
		});
		Ok(())
	}
}

impl<T: Config> CreateTokenInfo<T::AssetId, T::AccountId> for Module<T> {

	fn initial_amount(asset_id: &T::AssetId, account_id: &T::AccountId ) -> Self::Balance {
		Self::balances(&asset_id, account_id)
	}

	fn bal_conv(num: u128) -> Self::Balance { 
		num.saturated_into::<Self::Balance>()
	}
	
	fn exists(asset_id: &T::AssetId) -> bool {
		Self::token_infos(asset_id).is_some()
	}

	fn create_new_asset(token_info: TokenInfo< T::AccountId>, asset_id: T::AssetId) -> T::AssetId {
		<TokenInfos<T>>::insert(asset_id, &token_info);
		asset_id
	}

	fn mint(asset_id: &T::AssetId, who: &T::AccountId, value: Self::Balance) -> DispatchResult {
		ensure!(Self::exists(asset_id), Error::<T>::InvalidAsset);
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
		ensure!(Self::exists(asset_id), Error::<T>::InvalidAsset);
		let new_balance = Self::balances(asset_id, who)
			.checked_sub(&value)
			.ok_or(Error::<T>::InsufficientBalance)?;

		<Balances<T>>::mutate(asset_id, who, |balance| *balance = new_balance);
		<TotalSupply<T>>::mutate(asset_id, |supply| {
			*supply = supply.saturating_sub(value);
		});
		Self::deposit_event(RawEvent::Burn(asset_id.clone(), value));
		Ok(())
	} 
}
/*

pub struct GenesisConfig<T: Config> {
	pub endowed_accounts: Vec<(T::AccountId, T::AssetId, T::Balance)>,
}

#[cfg(feature = "std")]
impl<T: Config> Default for GenesisConfig<T> {
	fn default() -> Self {
		GenesisConfig {
			endowed_accounts: vec![],
		}
	}
}

impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
	fn build(&self) {
		// ensure no duplicates exist.
		let unique_endowed_accounts = self
			.endowed_accounts
			.iter()
			.map(|(account_id, currency_id, _)| (account_id, currency_id))
			.collect::<std::collections::BTreeSet<_>>();
		assert!(
			unique_endowed_accounts.len() == self.endowed_accounts.len(),
			"duplicate endowed accounts in genesis."
		);

		self.endowed_accounts
			.iter()
			.for_each(|(account_id, currency_id, initial_balance)| {
				assert!(
					*initial_balance >= T::ExistentialDeposits::get(&currency_id),
					"the balance of any account should always be more than existential deposit.",
				);
				CreateTokenInfo::<T>::mutate_account(account_id, *currency_id, |account_data, _| {
					account_data.free = *initial_balance
				});
				TotalIssuance::<T>::mutate(*currency_id, |total_issuance| {
					*total_issuance = total_issuance
						.checked_add(initial_balance)
						.expect("total issuance cannot overflow when building genesis")
				});
			});
	}
}
*/