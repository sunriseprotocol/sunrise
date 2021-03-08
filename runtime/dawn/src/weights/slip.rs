use frame_support::weights::{constants::RocksDbWeight as DbWeight, Weight};

use sp_std::marker::PhantomData;

pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> srs_pallet_slip::WeightInfo for WeightInfo<T> {
	fn mint() -> Weight {
		(71_000_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(6 as Weight))
	}
	fn redeem(strategy: &srs_pallet_slip::RedeemStrategy) -> Weight {
		match strategy {
			srs_pallet_slip::RedeemStrategy::Immediately => (88_000_000 as Weight)
				.saturating_add(DbWeight::get().reads(6 as Weight))
				.saturating_add(DbWeight::get().writes(5 as Weight)),
			srs_pallet_slip::RedeemStrategy::Target(_) => (75_000_000 as Weight)
				.saturating_add(DbWeight::get().reads(7 as Weight))
				.saturating_add(DbWeight::get().writes(5 as Weight)),
			srs_pallet_slip::RedeemStrategy::WaitForUnbonding => (47_000_000 as Weight)
				.saturating_add(DbWeight::get().reads(4 as Weight))
				.saturating_add(DbWeight::get().writes(4 as Weight)),
		}
	}
	fn withdraw_redemption() -> Weight {
		(53_000_000 as Weight)
			.saturating_add(DbWeight::get().reads(6 as Weight))
			.saturating_add(DbWeight::get().writes(2 as Weight))
	}
}
