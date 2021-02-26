Deliberately break the compilation here to validate the cargo files are correct

//! The Dev runtime. This can be compiled with `#[no_std]`, ready for Wasm.

#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
// The `large_enum_variant` warning originates from `construct_runtime` macro.
#![allow(clippy::large_enum_variant)]
#![allow(clippy::unnecessary_mut_passed)]
#![allow(clippy::or_fun_call)]
#![allow(clippy::from_over_into)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

use codec::Encode;
use hex_literal::hex;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
	crypto::KeyTypeId,
	u32_trait::{_1, _2, _3, _4},
	OpaqueMetadata, H160,
};
use sp_runtime::traits::{BadOrigin, BlakeTwo256, Block as BlockT, Convert, SaturatedConversion, StaticLookup};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{AccountIdConversion, Zero},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, DispatchResult, FixedPointNumber, ModuleId,
};
use sp_std::{collections::btree_set::BTreeSet, prelude::*};
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_system::{EnsureOneOf, EnsureRoot, RawOrigin};
use pallet_currencies::{BasicCurrencyAdapter, Currency};
use pallet_evm::{CallInfo, CreateInfo};
use pallet_evm_accounts::EvmAddressMapping;
use orml_traits::{create_median_value_data_provider, parameter_type_with_key, DataFeeder, DataProviderExtended};
// use pallet_grandpa::fg_primitives;
// use pallet_grandpa::{AuthorityId as GrandpaId, AuthorityList as
// GrandpaAuthorityList}; use pallet_session::historical as
// pallet_session_historical;
use pallet_transaction_payment::{FeeDetails, RuntimeDispatchInfo};

use cumulus_primitives_core::{relay_chain::Balance as RelayChainBalance, ParaId};
use orml_xcm_support::{CurrencyIdConverter, IsConcreteWithGeneralKey, MultiCurrencyAdapter, NativePalletAssetOr};
use polkadot_parachain::primitives::Sibling;
use xcm::v0::{Junction, MultiLocation, NetworkId};
use xcm_builder::{
	AccountId32Aliases, LocationInverter, ParentIsDefault, RelayChainAsNative, SiblingParachainAsNative,
	SiblingParachainConvertsVia, SignedAccountId32AsNative, SovereignSignedViaLocation,
};
use xcm_executor::{Config, XcmExecutor};

/// Weights for pallets used in the runtime.
mod weights;

pub use frame_support::{
	construct_runtime, debug, parameter_types,
	traits::{
		Contains, ContainsLengthBound, EnsureOrigin, Filter, Get, IsType, KeyOwnerProofSystem, LockIdentifier,
		Randomness, U128CurrencyToVote,
	},
	weights::{constants::RocksDbWeight, IdentityFee, Weight},
	StorageValue,
};

// pub use pallet_staking::StakerStatus;
pub use pallet_timestamp::Call as TimestampCall;
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Percent, Permill, Perquintill};

pub use authority::AuthorityConfigImpl;
pub use constants::{currency::*, fee::*, time::*};
pub use primitives::{
	AccountId, AccountIndex, AirDropCurrencyId, Amount, AuctionId, AuthoritysOriginId, Balance, BlockNumber,
	CurrencyId, DataProviderId, EraIndex, Hash, Moment, Nonce, Share, Signature, TokenSymbol, TradingPair,
};
pub use runtime_common::{
	BlockLength, BlockWeights, CurveFeeModel, ExchangeRate, GasToWeight, OffchainSolutionWeightLimit, Price, Rate,
	Ratio, SystemContractsFilter, TimeStampedPrice, AVERAGE_ON_INITIALIZE_RATIO,
};

mod authority;
mod benchmarking;
mod constants;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("dawn"),
	impl_name: create_runtime_str!("dawn"),
	authoring_version: 1,
	spec_version: 711,
	impl_version: 0,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
};

/// The version infromation used to identify this runtime when compiled
/// natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion {
		runtime_version: VERSION,
		can_author_with: Default::default(),
	}
}

impl_opaque_keys! {
	pub struct SessionKeys {
		// pub grandpa: Grandpa,
		// pub babe: Babe,
	}
}

// Module accounts of runtime
parameter_types! {
	pub const ExchangeModuleId: ModuleId = ModuleId(*b"exchange");
}

impl pallet_exchange::Config for Runtime {
	type Event = Event;
	type Currency = Currencies;
	type PoolId = u32;
	type PoolConfigId = u32;
	type Token = Tokens;
	type ModuleId = ExchangeModuleId;
	type TokenFunctions = Tokens;

}

parameter_types! {
	pub const BlockHashCount: BlockNumber = 900; // mortal tx can be valid up to 1 hour after signing
	pub const Version: RuntimeVersion = VERSION;
	pub const SS58Prefix: u8 = 42; // Ss58AddressFormat::SubstrateAccount
}

impl frame_system::Config for Runtime {
	type AccountId = AccountId;
	type Call = Call;
	type Lookup = (Indices, EvmAccounts);
	type Index = Nonce;
	type BlockNumber = BlockNumber;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type Header = generic::Header<BlockNumber, BlakeTwo256>;
	type Event = Event;
	type Origin = Origin;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = (
		module_evm::CallKillAccount<Runtime>,
		module_evm_accounts::CallKillAccount<Runtime>,
	);
	type DbWeight = RocksDbWeight;
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
}

// parameter_types! {
// 	pub const EpochDuration: u64 = EPOCH_DURATION_IN_SLOTS;
// 	pub const ExpectedBlockTime: Moment = MILLISECS_PER_BLOCK;
// }

// impl pallet_babe::Config for Runtime {
// 	type EpochDuration = EpochDuration;
// 	type ExpectedBlockTime = ExpectedBlockTime;
// 	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
// 	type KeyOwnerProofSystem = Historical;
// 	type KeyOwnerProof =
// 		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId,
// pallet_babe::AuthorityId)>>::Proof; 	type KeyOwnerIdentification =
// 		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId,
// pallet_babe::AuthorityId)>>::IdentificationTuple; 	type HandleEquivocation =
// pallet_babe::EquivocationHandler<Self::KeyOwnerIdentification, ()>; //
// Offences 	type WeightInfo = ();
// }

// impl pallet_grandpa::Config for Runtime {
// 	type Event = Event;
// 	type Call = Call;

// 	type KeyOwnerProofSystem = Historical;

// 	type KeyOwnerProof = <Self::KeyOwnerProofSystem as
// KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;

// 	type KeyOwnerIdentification =
// 		<Self::KeyOwnerProofSystem as KeyOwnerProofSystem<(KeyTypeId,
// GrandpaId)>>::IdentificationTuple;

// 	type HandleEquivocation =
// pallet_grandpa::EquivocationHandler<Self::KeyOwnerIdentification, ()>; //
// Offences

// 	type WeightInfo = ();
// }

parameter_types! {
	pub const IndexDeposit: Balance = DOLLARS;
}

impl pallet_indices::Config for Runtime {
	type AccountIndex = AccountIndex;
	type Event = Event;
	type Currency = Balances;
	type Deposit = IndexDeposit;
	type WeightInfo = ();
}

parameter_types! {
	pub const MinimumPeriod: u64 = SLOT_DURATION / 2;
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	// type OnTimestampSet = Babe;
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

// parameter_types! {
// 	pub const UncleGenerations: BlockNumber = 5;
// }

// impl pallet_authorship::Config for Runtime {
// 	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
// 	type UncleGenerations = UncleGenerations;
// 	type FilterUncle = ();
// 	type EventHandler = (Staking, ()); // ImOnline
// }

parameter_types! {
	pub const NativeTokenExistentialDeposit: Balance = 0;
	// For weight estimation, we assume that the most locks on an individual account will be 50.
	// This number may need to be adjusted in the future if this assumption no longer holds true.
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Runtime {
	type Balance = Balance;
	type DustRemoval = SunriseTreasury;
	type Event = Event;
	type ExistentialDeposit = NativeTokenExistentialDeposit;
	type AccountStore = frame_system::Module<Runtime>;
	type MaxLocks = MaxLocks;
	type WeightInfo = ();
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10 * MILLICENTS;
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(1, 100_000);
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000_000u128);
}

impl pallet_sudo::Config for Runtime {
	type Event = Event;
	type Call = Call;
}

impl pallet_utility::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_types! {
	pub const MultisigDepositBase: Balance = 500 * MILLICENTS;
	pub const MultisigDepositFactor: Balance = 100 * MILLICENTS;
	pub const MaxSignatories: u16 = 100;
}

impl pallet_multisig::Config for Runtime {
	type Event = Event;
	type Call = Call;
	type Currency = Balances;
	type DepositBase = MultisigDepositBase;
	type DepositFactor = MultisigDepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = ();
}


parameter_type_with_key! {
	pub ExistentialDeposits: |currency_id: CurrencyId| -> Balance {
		Zero::zero()
	};
}

impl orml_tokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type Amount = Amount;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Runtime, TreasuryModuleAccount>;
}

parameter_types! {
	pub const GetNativeCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::ACA);
	pub const GetStableCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::AUSD);
	pub const GetLDOTCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::LDOT);
}

impl module_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
	type WeightInfo = ();
	type AddressMapping = EvmAddressMapping<Runtime>;
	type EVMBridge = EVMBridge;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	Call: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: Call,
		public: <Signature as sp_runtime::traits::Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(
		Call,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		// take the biggest period possible.
		let period = BlockHashCount::get()
			.checked_next_power_of_two()
			.map(|c| c / 2)
			.unwrap_or(2) as u64;
		let current_block = System::block_number()
			.saturated_into::<u64>()
			// The `System::block_number` is initialized with `n+1`,
			// so the actual block number is `n`.
			.saturating_sub(1);
		let tip = 0;
		let extra: SignedExtra = (
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			module_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
			module_evm::SetEvmOrigin::<Runtime>::new(),
		);
		let raw_payload = SignedPayload::new(call, extra)
			.map_err(|e| {
				debug::warn!("Unable to create signed payload: {:?}", e);
			})
			.ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let address = Indices::unlookup(account);
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (address, signature, extra)))
	}
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as sp_runtime::traits::Verify>::Signer;
	type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
where
	Call: From<C>,
{
	type OverarchingCall = Call;
	type Extrinsic = UncheckedExtrinsic;
}

impl pallet_evm_accounts::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type KillAccount = frame_system::Consumer<Runtime>;
	type AddressMapping = EvmAddressMapping<Runtime>;
	type MergeAccount = Currencies;
	type WeightInfo = weights::evm_accounts::WeightInfo<Runtime>;
}

parameter_types! {
	pub const ChainId: u64 = 888;
	pub NetworkContractSource: H160 = H160::from_low_u64_be(0);
}

parameter_types! {
	pub const NewContractExtraBytes: u32 = 10_000;
	pub const StorageDepositPerByte: Balance = MICROCENTS;
	pub const MaxCodeSize: u32 = 60 * 1024;
	pub const DeveloperDeposit: Balance = DOLLARS;
	pub const DeploymentFee: Balance = DOLLARS;
}

pub type MultiCurrencyPrecompile =
	runtime_common::MultiCurrencyPrecompile<AccountId, EvmAddressMapping<Runtime>, Currencies>;

pub type StateRentPrecompile = runtime_common::StateRentPrecompile<AccountId, EvmAddressMapping<Runtime>, EVM>;
pub type ScheduleCallPrecompile = runtime_common::ScheduleCallPrecompile<
	AccountId,
	EvmAddressMapping<Runtime>,
	Scheduler,
	module_transaction_payment::ChargeTransactionPayment<Runtime>,
	Call,
	Origin,
	OriginCaller,
	Runtime,
>;

#[cfg(feature = "with-ethereum-compatibility")]
static ISTANBUL_CONFIG: evm::Config = evm::Config::istanbul();

impl module_evm::Config for Runtime {
	type AddressMapping = EvmAddressMapping<Runtime>;
	type Currency = Balances;
	type MergeAccount = Currencies;
	type NewContractExtraBytes = NewContractExtraBytes;
	type StorageDepositPerByte = StorageDepositPerByte;
	type MaxCodeSize = MaxCodeSize;

	type Event = Event;
	type Precompiles = runtime_common::AllPrecompiles<
		SystemContractsFilter,
		MultiCurrencyPrecompile,
		NFTPrecompile,
		StateRentPrecompile,
		OraclePrecompile,
		ScheduleCallPrecompile,
		DexPrecompile,
	>;
	type ChainId = ChainId;
	type GasToWeight = GasToWeight;
	type ChargeTransactionPayment = module_transaction_payment::ChargeTransactionPayment<Runtime>;
	type NetworkContractOrigin = EnsureRootOrTwoThirdsTechnicalCommittee;
	type NetworkContractSource = NetworkContractSource;
	type DeveloperDeposit = DeveloperDeposit;
	type DeploymentFee = DeploymentFee;
	type TreasuryAccount = TreasuryModuleAccount;
	type FreeDeploymentOrigin = EnsureRootOrHalfGeneralCouncil;
	type WeightInfo = weights::evm::WeightInfo<Runtime>;

	#[cfg(feature = "with-ethereum-compatibility")]
	fn config() -> &'static evm::Config {
		&ISTANBUL_CONFIG
	}
}

impl module_evm_bridge::Config for Runtime {
	type EVM = EVM;
}

#[cfg(not(feature = "standalone"))]
impl cumulus_pallet_parachain_system::Config for Runtime {
	type Event = Event;
	type OnValidationData = ();
	type SelfParaId = parachain_info::Module<Runtime>;
	type DownwardMessageHandlers = XcmHandler;
	type HrmpMessageHandlers = XcmHandler;
}

#[cfg(not(feature = "standalone"))]
impl parachain_info::Config for Runtime {}

#[cfg(not(feature = "standalone"))]
parameter_types! {
	pub const PolkadotNetworkId: NetworkId = NetworkId::Polkadot;
}

#[cfg(not(feature = "standalone"))]
pub struct AccountId32Convert;
#[cfg(not(feature = "standalone"))]
impl Convert<AccountId, [u8; 32]> for AccountId32Convert {
	fn convert(account_id: AccountId) -> [u8; 32] {
		account_id.into()
	}
}

#[cfg(not(feature = "standalone"))]
parameter_types! {
	pub SunriseNetwork: NetworkId = NetworkId::Named("sunrise".into());
	pub RelayChainOrigin: Origin = cumulus_pallet_xcm_handler::Origin::Relay.into();
	pub Ancestry: MultiLocation = MultiLocation::X1(Junction::Parachain {
		id: ParachainInfo::get().into(),
	});
	pub const RelayChainCurrencyId: CurrencyId = CurrencyId::Token(TokenSymbol::DOT);
}

#[cfg(not(feature = "standalone"))]
pub type LocationConverter = (
	ParentIsDefault<AccountId>,
	SiblingParachainConvertsVia<Sibling, AccountId>,
	AccountId32Aliases<SunriseNetwork, AccountId>,
);

#[cfg(not(feature = "standalone"))]
pub type LocalAssetTransactor = MultiCurrencyAdapter<
	Currencies,
	IsConcreteWithGeneralKey<CurrencyId, RelayToNative>,
	LocationConverter,
	AccountId,
	CurrencyIdConverter<CurrencyId, RelayChainCurrencyId>,
	CurrencyId,
>;

#[cfg(not(feature = "standalone"))]
pub type LocalOriginConverter = (
	SovereignSignedViaLocation<LocationConverter, Origin>,
	RelayChainAsNative<RelayChainOrigin, Origin>,
	SiblingParachainAsNative<cumulus_pallet_xcm_handler::Origin, Origin>,
	SignedAccountId32AsNative<SunriseNetwork, Origin>,
);

#[cfg(not(feature = "standalone"))]
parameter_types! {
	pub NativeOrmlTokens: BTreeSet<(Vec<u8>, MultiLocation)> = {
		let mut t = BTreeSet::new();
		//TODO: might need to add other assets based on orml-tokens

		// Plasm
		t.insert(("SDN".into(), (Junction::Parent, Junction::Parachain { id: 5000 }).into()));
		// Plasm
		t.insert(("PLM".into(), (Junction::Parent, Junction::Parachain { id: 5000 }).into()));
		t
	};
}

#[cfg(not(feature = "standalone"))]
pub struct XcmConfig;
#[cfg(not(feature = "standalone"))]
impl Config for XcmConfig {
	type Call = Call;
	type XcmSender = XcmHandler;
	type AssetTransactor = LocalAssetTransactor;
	type OriginConverter = LocalOriginConverter;
	//TODO: might need to add other assets based on orml-tokens
	type IsReserve = NativePalletAssetOr<NativeOrmlTokens>;
	type IsTeleporter = ();
	type LocationInverter = LocationInverter<Ancestry>;
}

#[cfg(not(feature = "standalone"))]
impl cumulus_pallet_xcm_handler::Config for Runtime {
	type Event = Event;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type UpwardMessageSender = ParachainSystem;
	type HrmpMessageSender = ParachainSystem;
}

#[cfg(not(feature = "standalone"))]
pub struct RelayToNative;
#[cfg(not(feature = "standalone"))]
impl Convert<RelayChainBalance, Balance> for RelayToNative {
	fn convert(val: u128) -> Balance {
		// native is 18
		// relay is 12
		val * 1_000_000
	}
}

#[cfg(not(feature = "standalone"))]
pub struct NativeToRelay;
#[cfg(not(feature = "standalone"))]
impl Convert<Balance, RelayChainBalance> for NativeToRelay {
	fn convert(val: u128) -> Balance {
		// native is 18
		// relay is 12
		val / 1_000_000
	}
}

#[cfg(not(feature = "standalone"))]
impl orml_xtokens::Config for Runtime {
	type Event = Event;
	type Balance = Balance;
	type ToRelayChainBalance = NativeToRelay;
	type AccountId32Convert = AccountId32Convert;
	//TODO: change network id if kusama
	type RelayChainNetworkId = PolkadotNetworkId;
	type ParaId = ParachainInfo;
	type AccountIdConverter = LocationConverter;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

macro_rules! construct_dawn_runtime {
	($( $modules:tt )*) => {
		#[allow(clippy::large_enum_variant)]
		construct_runtime! {
			pub enum Runtime where
				Block = Block,
				NodeBlock = primitives::Block,
				UncheckedExtrinsic = UncheckedExtrinsic
			{
				// Core
				System: frame_system::{Module, Call, Storage, Config, Event<T>},
				Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
				RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},

				// Tokens & Related
				Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},

				TransactionPayment: module_transaction_payment::{Module, Call, Storage},
				EvmAccounts: module_evm_accounts::{Module, Call, Storage, Event<T>},
				Currencies: module_currencies::{Module, Call, Event<T>},
				Tokens: orml_tokens::{Module, Storage, Event<T>, Config<T>},

				// Utility
				Utility: pallet_utility::{Module, Call, Event},
				Multisig: pallet_multisig::{Module, Call, Storage, Event<T>},
				Recovery: pallet_recovery::{Module, Call, Storage, Event<T>},
				Proxy: pallet_proxy::{Module, Call, Storage, Event<T>},

				Indices: pallet_indices::{Module, Call, Storage, Config<T>, Event<T>},

				EVM: module_evm::{Module, Config<T>, Call, Storage, Event<T>},
				EVMBridge: module_evm_bridge::{Module},

				$($modules)*

				// Dev
				Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
			}
		}
	}
}

#[cfg(not(feature = "standalone"))]
construct_dawn_runtime! {
	// Parachain
	ParachainSystem: cumulus_pallet_parachain_system::{Module, Call, Storage, Inherent, Event},
	ParachainInfo: parachain_info::{Module, Storage, Config},
	XcmHandler: cumulus_pallet_xcm_handler::{Module, Call, Event<T>, Origin},
	XTokens: orml_xtokens::{Module, Storage, Call, Event<T>},
}

#[cfg(feature = "standalone")]
construct_dawn_runtime! {}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, AccountIndex>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;
/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	module_transaction_payment::ChargeTransactionPayment<Runtime>,
	module_evm::SetEvmOrigin<Runtime>,
);
/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<Address, Call, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<Call, SignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic = generic::CheckedExtrinsic<AccountId, Call, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive =
	frame_executive::Executive<Runtime, Block, frame_system::ChainContext<Runtime>, Runtime, AllModules>;

#[cfg(not(feature = "disable-runtime-api"))]
impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block)
		}

		fn initialize_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			Runtime::metadata().into()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}

		fn random_seed() -> <Block as BlockT>::Hash {
			RandomnessCollectiveFlip::random_seed()
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<
		Block,
		Balance,
	> for Runtime {
		fn query_info(uxt: <Block as BlockT>::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(uxt: <Block as BlockT>::Extrinsic, len: u32) -> FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}

	impl module_evm_rpc_runtime_api::EVMRuntimeRPCApi<Block, Balance> for Runtime {
		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u32,
			storage_limit: u32,
			estimate: bool,
		) -> Result<CallInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as module_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			module_evm::Runner::<Runtime>::call(
				from,
				from,
				to,
				data,
				value,
				gas_limit.into(),
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as module_evm::Config>::config()),
			)
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: Balance,
			gas_limit: u32,
			storage_limit: u32,
			estimate: bool,
		) -> Result<CreateInfo, sp_runtime::DispatchError> {
			let config = if estimate {
				let mut config = <Runtime as module_evm::Config>::config().clone();
				config.estimate = true;
				Some(config)
			} else {
				None
			};

			module_evm::Runner::<Runtime>::create(
				from,
				data,
				value,
				gas_limit.into(),
				storage_limit,
				config.as_ref().unwrap_or(<Runtime as module_evm::Config>::config()),
			)
		}
	}

	// benchmarks for sunrise modules
	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{Benchmarking, BenchmarkBatch, add_benchmark, TrackedStorageKey};
			use orml_benchmarking::{add_benchmark as orml_add_benchmark};

			use module_nft_benchmarking::Module as NftBench;
			impl module_nft_benchmarking::Config for Runtime {}

			let whitelist: Vec<TrackedStorageKey> = vec![
				// Block Number
				// frame_system::Number::<Runtime>::hashed_key().to_vec(),
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef702a5c1b19ab7a04f536c519aca4983ac").to_vec().into(),
				// Total Issuance
				hex_literal::hex!("c2261276cc9d1f8598ea4b6a74b15c2f57c875e4cff74148e4628f264b974c80").to_vec().into(),
				// Execution Phase
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7ff553b5a9862a516939d82b3d3d8661a").to_vec().into(),
				// Event Count
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef70a98fdbe9ce6c55837576c60c7af3850").to_vec().into(),
				// System Events
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef780d41e5e16056765bc8461851072c9d7").to_vec().into(),
				// Caller 0 Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da946c154ffd9992e395af90b5b13cc6f295c77033fce8a9045824a6690bbf99c6db269502f0a8d1d2a008542d5690a0749").to_vec().into(),
				// Treasury Account
				hex_literal::hex!("26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da95ecffd7b6c0f78751baa9d281e0bfa3a6d6f646c70792f74727372790000000000000000000000000000000000000000").to_vec().into(),
			];
			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);

			add_benchmark!(params, batches, nft, NftBench::<Runtime>);
			orml_add_benchmark!(params, batches, dex, benchmarking::dex);
			orml_add_benchmark!(params, batches, auction_manager, benchmarking::auction_manager);
			orml_add_benchmark!(params, batches, cdp_engine, benchmarking::cdp_engine);
			orml_add_benchmark!(params, batches, emergency_shutdown, benchmarking::emergency_shutdown);
			orml_add_benchmark!(params, batches, evm, benchmarking::evm);
			orml_add_benchmark!(params, batches, honzon, benchmarking::honzon);
			orml_add_benchmark!(params, batches, cdp_treasury, benchmarking::cdp_treasury);
			orml_add_benchmark!(params, batches, transaction_payment, benchmarking::transaction_payment);
			orml_add_benchmark!(params, batches, incentives, benchmarking::incentives);
			orml_add_benchmark!(params, batches, prices, benchmarking::prices);
			orml_add_benchmark!(params, batches, evm_accounts, benchmarking::evm_accounts);
			orml_add_benchmark!(params, batches, homa, benchmarking::homa);

			orml_add_benchmark!(params, batches, orml_tokens, benchmarking::tokens);
			orml_add_benchmark!(params, batches, orml_vesting, benchmarking::vesting);
			orml_add_benchmark!(params, batches, orml_auction, benchmarking::auction);
			orml_add_benchmark!(params, batches, module_currencies, benchmarking::currencies);

			orml_add_benchmark!(params, batches, orml_authority, benchmarking::authority);
			orml_add_benchmark!(params, batches, orml_gradually_update, benchmarking::gradually_update);
			orml_add_benchmark!(params, batches, orml_rewards, benchmarking::rewards);
			orml_add_benchmark!(params, batches, orml_oracle, benchmarking::oracle);

			if batches.is_empty() { return Err("Benchmark not found for this module.".into()) }
			Ok(batches)
		}
	}
}

#[cfg(not(feature = "standalone"))]
cumulus_pallet_parachain_system::register_validate_block!(Block, Executive);

#[cfg(test)]
mod tests {
	use super::*;
	use frame_system::offchain::CreateSignedTransaction;

	#[test]
	fn validate_transaction_submitter_bounds() {
		fn is_submit_signed_transaction<T>()
		where
			T: CreateSignedTransaction<Call>,
		{
		}

		is_submit_signed_transaction::<Runtime>();
	}
}

#[test]
fn transfer() {
	let t = Call::System(frame_system::Call::remark(vec![1, 2, 3])).encode();
	println!("t: {:?}", t);
}
