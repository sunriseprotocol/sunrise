//! Unit tests for the airdrop module.

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::{Airdrop, Event, ExtBuilder, Origin, System, SRS, ALICE, BOB, CHARLIE, BEAM};
use sp_runtime::traits::BadOrigin;

#[test]
fn airdrop_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_noop!(Airdrop::airdrop(Origin::signed(BOB), ALICE, BEAM, 10000), BadOrigin,);
		assert_ok!(Airdrop::airdrop(Origin::root(), ALICE, BEAM, 10000));
		let airdrop_event = Event::airdrop(RawEvent::Airdrop(ALICE, BEAM, 10000));
		assert!(System::events().iter().any(|record| record.event == airdrop_event));
		assert_eq!(Airdrop::airdrops(ALICE, BEAM), 10000);
	});
}

#[test]
fn update_airdrop_work() {
	ExtBuilder::default().build().execute_with(|| {
		System::set_block_number(1);
		assert_ok!(Airdrop::airdrop(Origin::root(), ALICE, SRS, 10000));
		assert_ok!(Airdrop::airdrop(Origin::root(), ALICE, SRS, 10000));
		assert_eq!(Airdrop::airdrops(ALICE, SRS), 20000);
		assert_noop!(Airdrop::update_airdrop(Origin::signed(BOB), ALICE, SRS, 0), BadOrigin,);
		assert_ok!(Airdrop::update_airdrop(Origin::root(), ALICE, SRS, 0));
		let update_airdrop_event = Event::airdrop(RawEvent::UpdateAirdrop(ALICE, SRS, 0));
		assert!(System::events()
			.iter()
			.any(|record| record.event == update_airdrop_event));
		assert_eq!(Airdrop::airdrops(ALICE, SRS), 0);
	});
}

#[test]
fn genesis_config_work() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(Airdrop::airdrops(CHARLIE, BEAM), 150);
		assert_eq!(Airdrop::airdrops(CHARLIE, SRS), 80);
	});
}
