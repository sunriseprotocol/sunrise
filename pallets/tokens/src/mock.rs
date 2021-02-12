use crate::{Module, Config};
use sp_core::H256;
use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{
	traits::{ IdentityLookup}, testing::Header, Perbill,
};
use frame_system as system;

use primitives::{Balance};

mod tokens {
	pub use super::super::*;
}

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

impl_outer_event! {
	pub enum TestEvent for Runtime {
		frame_system<T>,
		tokens<T>,
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

impl Config for Runtime {
	type Event = TestEvent;
	type Balance = Balance;
	type AssetId = u128;
}

pub type AccountId = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;


pub type System = frame_system::Module<Runtime>;
pub type SRSTokens = Module<Runtime>;

pub fn test_environment() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into()
}
