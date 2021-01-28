use crate::{Module, Trait};
use sp_core::H256;
use frame_support::{impl_outer_origin, parameter_types, weights::Weight};
use sp_runtime::{  
	traits::{BlakeTwo256, IdentityLookup },testing::Header, Perbill, ModuleId 
};
use frame_system as system;
use pallet_srstokens::{Token, CreateTokenInfo};
use orml_traits::{MultiReservableCurrency, MultiCurrency};

impl_outer_origin! {
	pub enum Origin for Runtime {}
}

#[derive(Clone, Eq, PartialEq)]
pub struct Runtime;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: Weight = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
}

impl system::Trait for Runtime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Call = ();
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type DbWeight = ();
	type BlockExecutionWeight = ();
	type ExtrinsicBaseWeight = ();
	type MaximumExtrinsicWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
	type PalletInfo = ();
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
}


pub type SRSTokens = pallet_srstokens::Module<Runtime>;
pub type Balance = u128;
pub type CurrencyId = u64;

parameter_types! {
    pub const ModId: ModuleId = ModuleId(*b"exchange");

}

impl Trait for Runtime {
    type Event = ();
    type Currency = MultiReservableCurrency<AccountId>: MultiCurrency<AccountId>;
    type PoolId = u64;
    type PoolConfigId = u64;
	type Balance = Balance;
    type Token = SRSTokens::Token<AssetId, AccountId>;
	type ModuleId = ModId;
	type TokenFunctions = SRSTokens::CreateTokenInfo<AssetId, AccountId>;
    
}



pub type AccountId = u64;
pub type AssetId = u64;

pub const ALICE: AccountId = 1;
    pub const BOB: AccountId = 2;

    pub type System = frame_system::Module<Runtime>;
pub type Exchange = Module<Runtime>;

pub fn test_environment() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Runtime>().unwrap().into()
}
