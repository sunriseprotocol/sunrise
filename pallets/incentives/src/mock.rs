//! Mocks for the incentives module.

#![cfg(test)]

use super::*;
use frame_support::{
	construct_runtime,
	dispatch::{DispatchError, DispatchResult},
	ord_parameter_types, parameter_types,
};
use frame_system::EnsureSignedBy;
use orml_traits::parameter_type_with_key;
use srs_primitives::TokenSymbol;
use sp_core::H256;
use sp_runtime::{testing::Header, traits::IdentityLookup};
use sp_std::cell::RefCell;
pub use srs_pallet_support::{SHYTreasury, DEXManager, Price, Ratio};

pub type AccountId = u128;
pub type BlockNumber = u64;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const SRS: CurrencyId = CurrencyId::Token(TokenSymbol::SRS);
pub const SUSD: CurrencyId = CurrencyId::Token(TokenSymbol::SUSD);
pub const BTC: CurrencyId = CurrencyId::Token(TokenSymbol::XBTC);
pub const DOT: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
pub const BTC_SUSD_LP: CurrencyId = CurrencyId::DEXShare(TokenSymbol::XBTC, TokenSymbol::SUSD);
pub const DOT_SUSD_LP: CurrencyId = CurrencyId::DEXShare(TokenSymbol::DOT, TokenSymbol::SUSD);

mod incentives {
	pub use super::super::*;
}

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = ();
}

pub struct MockSHYTreasury;
impl SHYTreasury<AccountId> for MockSHYTreasury {
	type Balance = Balance;
	type CurrencyId = CurrencyId;

	fn get_surplus_pool() -> Balance {
		unimplemented!()
	}

	fn get_debit_pool() -> Balance {
		unimplemented!()
	}

	fn get_total_collaterals(_: CurrencyId) -> Balance {
		unimplemented!()
	}

	fn get_debit_proportion(_: Balance) -> Ratio {
		unimplemented!()
	}

	fn on_system_debit(_: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn on_system_surplus(_: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn issue_debit(who: &AccountId, debit: Balance, _: bool) -> DispatchResult {
		TokensModule::deposit(SRS, who, debit)
	}

	fn burn_debit(_: &AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn deposit_surplus(_: &AccountId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn deposit_collateral(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		unimplemented!()
	}

	fn withdraw_collateral(_: &AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		unimplemented!()
	}
}

pub struct MockDEX;
impl DEXManager<AccountId, CurrencyId, Balance> for MockDEX {
	fn get_liquidity_pool(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> (Balance, Balance) {
		match (currency_id_a, currency_id_b) {
			(SUSD, BTC) => (500, 100),
			(SUSD, DOT) => (400, 100),
			(BTC, SUSD) => (100, 500),
			(DOT, SUSD) => (100, 400),
			_ => (0, 0),
		}
	}

	fn get_swap_target_amount(_: &[CurrencyId], _: Balance, _: Option<Ratio>) -> Option<Balance> {
		unimplemented!()
	}

	fn get_swap_supply_amount(_: &[CurrencyId], _: Balance, _: Option<Ratio>) -> Option<Balance> {
		unimplemented!()
	}

	fn swap_with_exact_supply(
		_: &AccountId,
		_: &[CurrencyId],
		_: Balance,
		_: Balance,
		_: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}

	fn swap_with_exact_target(
		_: &AccountId,
		_: &[CurrencyId],
		_: Balance,
		_: Balance,
		_: Option<Ratio>,
	) -> sp_std::result::Result<Balance, DispatchError> {
		unimplemented!()
	}
}

thread_local! {
	static IS_SHUTDOWN: RefCell<bool> = RefCell::new(false);
}

pub fn mock_shutdown() {
	IS_SHUTDOWN.with(|v| *v.borrow_mut() = true)
}

pub struct MockEmergencyShutdown;
impl EmergencyShutdown for MockEmergencyShutdown {
	fn is_shutdown() -> bool {
		IS_SHUTDOWN.with(|v| *v.borrow_mut())
	}
}

impl orml_rewards::Config for Runtime {
	type Share = Balance;
	type Balance = Balance;
	type PoolId = PoolId;
	type Handler = IncentivesModule;
	type WeightInfo = ();
}

parameter_types! {
	pub const LoansIncentivePool: AccountId = 10;
	pub const DexIncentivePool: AccountId = 11;
	pub const HomaIncentivePool: AccountId = 12;
	pub const AccumulatePeriod: BlockNumber = 10;
	pub const IncentiveCurrencyId: CurrencyId = SRS;
	pub const SavingCurrencyId: CurrencyId = SUSD;
	pub const IncentivesModuleId: ModuleId = ModuleId(*b"srs/inct");
}

ord_parameter_types! {
	pub const Four: AccountId = 4;
}

impl Config for Runtime {
	type Event = Event;
	type LoansIncentivePool = LoansIncentivePool;
	type DexIncentivePool = DexIncentivePool;
	type HomaIncentivePool = HomaIncentivePool;
	type AccumulatePeriod = AccumulatePeriod;
	type IncentiveCurrencyId = IncentiveCurrencyId;
	type SavingCurrencyId = SavingCurrencyId;
	type UpdateOrigin = EnsureSignedBy<Four, AccountId>;
	type SHYTreasury = MockSHYTreasury;
	type Currency = TokensModule;
	type DEX = MockDEX;
	type EmergencyShutdown = MockEmergencyShutdown;
	type ModuleId = IncentivesModuleId;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Storage, Config, Event<T>},
		IncentivesModule: incentives::{Module, Storage, Call, Event<T>},
		TokensModule: orml_tokens::{Module, Storage, Event<T>},
		RewardsModule: orml_rewards::{Module, Storage, Call},
	}
);

#[derive(Default)]
pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		t.into()
	}
}
