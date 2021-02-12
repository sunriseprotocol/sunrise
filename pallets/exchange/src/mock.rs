use crate::{Module, Config};
use sp_core::H256;
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{  
	traits::{ IdentityLookup },testing::Header, Perbill, ModuleId 
};
use frame_system as system;
use orml_traits::{parameter_type_with_key};

use primitives::{Balance, CurrencyId};

mod exchange {
	pub use super::super::*;
}
impl_outer_origin! {
	pub enum Origin for Runtime {}
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		frame_system<T>,
		exchange<T>,
		orml_tokens<T>,
		pallet_tokens<T>,
	}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Runtime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = ();
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = TestEvent;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
}


pub type OrmlTokens = orml_tokens::Module<Runtime>;



impl pallet_tokens::Config for Runtime {
	type Event = TestEvent;
	type Balance = u128;
	type AssetId = u64;
}

pub type Tokens = pallet_tokens::Module<Runtime>;


parameter_types! {
    pub const ModId: ModuleId = ModuleId(*b"exchange");

}

impl Config for Runtime {
	type Event = TestEvent;
    type Currency = OrmlTokens;
    type PoolId = u64;
    type PoolConfigId = u64;
	type ModuleId = ModId;
	type Token = Tokens;
	type TokenFunctions = Tokens;
}

pub type Amount = i128;


pub type AccountId = u64;
pub type AssetId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const ACCOUNTS: [AccountId; 2] = [ALICE, BOB];

pub type System = frame_system::Module<Runtime>;
pub type Exchange = Module<Runtime>;

pub fn test_environment() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into()
}

pub struct ExtBuilder {
	token_init:  Vec<(AccountId, AssetId, Balance, Vec<u8>,  Vec<AccountId>, Balance)>,
}

impl Default for ExtBuilder {

	fn default() -> Self {
		Self {
			token_init: vec![
				(ALICE, 3_u64, 1_000_000_000_u128, (*b"ACA").to_vec(), ACCOUNTS.to_vec().clone(), 1_000_000_u128  ),
				(ALICE, 4_u64, 2_000_000_000_u128, (*b"XBTC").to_vec(), ACCOUNTS.to_vec().clone(), 1_000_000_u128   ),
			],
		}
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

			pallet_tokens::GenesisConfig::<Runtime> {
			token_init: self.token_init,
		}
		.assimilate_storage(&mut t)
		.unwrap();

		t.into()
	}
}