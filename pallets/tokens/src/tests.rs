use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

type AssetId = u128;
type Balance = u128;

#[test]
fn create_asset() {
	test_environment().execute_with(|| {
		System::set_block_number(1);
		let wrapped_srs: Vec<u8> = (*b"Wrapped_SRS").to_vec();
		let wrapped_sym: Vec<u8> = (*b"wSRS").to_vec();
		let asset_id: AssetId = 1_000_u128;
		let initial_bal: Balance = 1_000_000_u128;
		let empty_bal: Balance = 0_u128;

		//create token and mint
		assert_ok!(SRSTokens::create_new_asset(Origin::signed(1), wrapped_srs, 18, wrapped_sym, ALICE, initial_bal,asset_id));
		//ensure token meta data is good
		assert_eq!(
			(*b"Wrapped_SRS").to_vec(), SRSTokens::token_infos(asset_id).unwrap().name);
		assert_eq!(
			(*b"wSRS").to_vec(), SRSTokens::token_infos(asset_id).unwrap().symbol);
		assert_eq!(
			18, SRSTokens::token_infos(asset_id).unwrap().decimals);
		assert_eq!(
			ALICE, SRSTokens::token_infos(asset_id).unwrap().owner);

		//ensure Alice has tokens and bob doesnt
		assert_eq!(SRSTokens::balances(asset_id, ALICE), initial_bal);
		assert_eq!(SRSTokens::balances(asset_id, BOB), empty_bal);
		assert_eq!(SRSTokens::total_supply(asset_id), initial_bal);

		//test successful transfer
		let sent_bal: Balance = 50_000_u128;
		assert_ok!(SRSTokens::transfer(Origin::signed(1), asset_id, BOB, sent_bal));
		assert_eq!(SRSTokens::balances(asset_id, ALICE), initial_bal - sent_bal);
		assert_eq!(SRSTokens::balances(asset_id, BOB), sent_bal);

		//test bad transfer
		assert_noop!(
			SRSTokens::transfer(Origin::signed(1), asset_id, BOB, initial_bal),
			Error::<Runtime>::InsufficientBalance
		);

		//transfer on non existant token
		let bad_asset_id: AssetId = 5_000_u128;
		assert_noop!(
			SRSTokens::transfer(Origin::signed(1), bad_asset_id, BOB, sent_bal),
			Error::<Runtime>::InvalidAsset
		);

		//recreate existing token
		assert_noop!(
			SRSTokens::create_new_asset(Origin::signed(1), (*b"Wrapped_SRS").to_vec(), 18, (*b"wSRS").to_vec(), BOB, initial_bal,asset_id)
			, Error::<Runtime>::InvalidIdGeneration);

	});
}

