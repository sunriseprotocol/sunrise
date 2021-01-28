use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use orml_traits::{MultiReservableCurrency};
use sp_runtime::{ModuleId};

type AssetId = u128;
type Balance = u128;
type PoolConfigId = u32;
type PoolId = u32;

#[test]
fn create_asset() {
	test_environment().execute_with(|| {
        System::set_block_number(1);
        let pool_id: u32 = 1_000_u32;
        let asset_id_one: AssetId = 1_000_u128;
		let asset_id_one: AssetId = 2_000_u128;
        let pool_config_id: PoolConfigId = 120_u32;
        assert_ok!(Exchange::liquidity_config_creation(Origin::signed(1), 120_u32, vec![1_000_u128, 2_000_u128], pool_config_id, vec![1_u64, 2_u64], 0_32, 1_u128, 1_u128, 1_u128, 1_u128, 2_u8  ));
        
    });
}

