use primitives::{Balance};
use sp_std::convert::{TryFrom};

const ONE: Balance = 1;

pub fn math_price(balance_in: Balance, weight_in: u128, balance_out: Balance, weight_out: u128, fees: Balance ) -> Balance {

	let numer = balance_in.checked_div(weight_in);
    let denom = balance_out.checked_div(weight_out);
	let ratio = numer.unwrap().checked_div(denom.unwrap());
    let scale = ONE.checked_div(ONE.checked_sub(fees).unwrap());
    ratio.unwrap().checked_mul(scale.unwrap()).unwrap()
}

pub fn math_swap_exact_in(balance_in: Balance, weight_in: u128, balance_out: Balance, weight_out: u128, token_amount_in: Balance,  fees: Balance) -> Balance {

    let weight_ratio: u32 = u32::try_from(weight_in.checked_div(weight_out).unwrap()).unwrap();
    let mut adjusted_in = ONE.checked_sub( fees);
    adjusted_in = token_amount_in.checked_mul(adjusted_in.unwrap());
    let y = balance_in.checked_div(balance_in.checked_add(adjusted_in.unwrap()).unwrap());
    let foo = y.unwrap().checked_pow(weight_ratio);
    let bar = ONE.checked_sub( foo.unwrap());
    balance_out.checked_mul(bar.unwrap()).unwrap()
}

pub fn math_swap_exact_out(balance_in: Balance, weight_in: u128, balance_out: Balance, weight_out: u128, token_amount_out: Balance, fees: Balance  ) -> Balance {
		
    let weight_ratio: u32 = u32::try_from(weight_in.checked_div(weight_out).unwrap()).unwrap();
    let diff = balance_out.checked_sub( token_amount_out);
    let y = balance_out.checked_div(diff.unwrap());
    let mut foo = y.unwrap().checked_pow(weight_ratio);
    foo = foo.unwrap().checked_sub(ONE);
    let mut token_amount_in = ONE.checked_sub(fees);
    token_amount_in = (balance_in.checked_mul(foo.unwrap())).unwrap().checked_div(token_amount_in.unwrap());
    token_amount_in.unwrap()
}

pub fn math_join_pool_with_min_lptokens_given(balance_in: Balance, weight_in: u128, pool_supply: Balance, total_weight: u128, token_amount_in: Balance, fees: Balance) -> Balance {    //calcPoolOutGivenSingleIn 
    
    let normalized_weight: u32 = u32::try_from(weight_in.checked_div(total_weight).unwrap()).unwrap();
    let zaz = (ONE.checked_sub(normalized_weight.into())).unwrap().checked_mul(fees); 
    let token_amount_in_after_fee = token_amount_in.checked_mul(ONE.checked_sub( zaz.unwrap()).unwrap());
    let new_token_balance_in = balance_in.checked_add( token_amount_in_after_fee.unwrap());
    let token_in_ratio = new_token_balance_in.unwrap().checked_div( balance_in);
    let pool_ratio = token_in_ratio.unwrap().checked_pow(normalized_weight);
    let new_pool_supply = pool_ratio.unwrap().checked_mul( pool_supply);
    new_pool_supply.unwrap().checked_sub( pool_supply).unwrap()
}

pub fn math_join_pool_with_max_collateral_taken(balance_in: Balance, weight_in: u128, pool_supply: Balance, total_weight: u128, pool_amount_out: Balance, fees: Balance) -> Balance {      //calcSingleInGivenPoolOut
    
    let normalized_weight = weight_in.checked_div(total_weight);
    let new_pool_supply = pool_supply.checked_add(pool_amount_out);
    let pool_ratio = new_pool_supply.unwrap().checked_div(pool_supply);
    let boo: u32 = u32::try_from(  ONE.checked_div(normalized_weight.unwrap()).unwrap()).unwrap(); 
    let token_in_ratio = pool_ratio.unwrap().checked_pow( boo);
    let new_token_balance_in = token_in_ratio.unwrap().checked_mul( balance_in);
    let token_amount_in_after_fee = new_token_balance_in.unwrap().checked_sub(balance_in);
    let zar = (ONE.checked_sub(normalized_weight.unwrap()).unwrap()).checked_mul(fees);
    token_amount_in_after_fee.unwrap().checked_div( ONE.checked_sub(zar.unwrap()).unwrap()).unwrap()

}

pub fn math_exit_pool_with_min_collateral_received(balance_out: Balance, weight_out: u128, pool_supply: Balance, total_weight: u128, pool_amount_in: Balance, fees: Balance) -> Balance {        //calcSingleOutGivenPoolIn
   
    let normalized_weight: u32 = u32::try_from(weight_out.checked_div(total_weight).unwrap()).unwrap();
    let pool_amount_in_after_exit_fee = pool_amount_in.checked_mul( ONE.checked_sub( 0_u128).unwrap());
    let new_pool_supply = pool_supply.checked_sub( pool_amount_in_after_exit_fee.unwrap());
    let pool_ratio = new_pool_supply.unwrap().checked_div( pool_supply);
    let token_out_ratio = pool_ratio.unwrap().checked_pow( safe_one().checked_div( normalized_weight.into()).unwrap());
    let new_token_balance_out = token_out_ratio.unwrap().checked_mul( balance_out);
    let token_amount_out_before_swap_fee = balance_out.checked_sub( new_token_balance_out.unwrap());
    let zaz = (ONE.checked_sub( normalized_weight.into())).unwrap().checked_mul( fees); 
    token_amount_out_before_swap_fee.unwrap().checked_mul(ONE.checked_sub( zaz.unwrap()).unwrap()).unwrap()
}

pub fn math_exit_pool_with_max_lp_given(balance_out: Balance, weight_out: u128, pool_supply: Balance, total_weight: u128, token_amount_out: Balance, fees: Balance) -> Balance	{	     //calcPoolInGivenSingleOut

    let normalized_weight: u32 = u32::try_from(weight_out.checked_div(total_weight).unwrap()).unwrap();
    let zoo = ONE.checked_sub( normalized_weight.into());
    let zar = zoo.unwrap().checked_mul( fees); 
    let token_amount_out_before_swap_fee = token_amount_out.checked_div(ONE.checked_sub( zar.unwrap()).unwrap());
    let new_token_balance_out = balance_out.checked_sub(token_amount_out_before_swap_fee.unwrap());
    let token_out_ratio = new_token_balance_out.unwrap().checked_div( balance_out);
    let pool_ratio = token_out_ratio.unwrap().checked_pow( normalized_weight);
    let new_pool_supply = pool_ratio.unwrap().checked_mul( pool_supply);
    let pool_amount_in_after_exit_fee = pool_supply.checked_sub( new_pool_supply.unwrap());
    pool_amount_in_after_exit_fee.unwrap().checked_div( ONE.checked_sub( 0_u128).unwrap()).unwrap()
}

fn safe_one()-> u32{ 
    let one: u32 = u32::try_from(ONE).unwrap();
    one
}