use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};
use sp_runtime::{ModuleId};

type AssetId = u64;
type Balance = u128;
type PoolConfigId = u64;
type PoolId = u64;

#[test]
fn create_asset() {
	ExtBuilder::default()
		.build()
		.execute_with(|| {
        System::set_block_number(1);
        let pool_id: u64 = 1_000_u64;
        let pool_config_id: PoolConfigId = 130_u64;
        assert_ok!(Exchange::liquidity_config_creation(Origin::signed(1), 130_u32, vec![3_u64, 4_u64], vec![1_000_u64, 1_000_u64], 0_32, 1_u128, 1_u128, 1_u128, 1_u32, 2_u8  ));
        assert_ok!(Exchange::liquidity_pool_create(Origin::signed(1), 130_u32, vec![3_u64, 4_u64], pool_config_id, vec![1_000_u128, 2_000_u128], ALICE, 1_u64  ));
        
    });
}

