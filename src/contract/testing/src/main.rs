use std::cmp::min;
use uint::construct_uint;
use near_sdk::{Balance, Gas};
construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

const TGAS_DIVIDER: u64 = 1_000_000_000_000;

fn main() {
    let amounts = vec![445236146982962_u128, 48009037553277982350_u128];
    //                  usdc             wnear
    let total_amount = vec![Balance::from(1299999618200067583682_u128), Balance::from(129999961727999868694692022_u128)];
    let shares_total_supply = 13000000000000000000000000_u128;

    let mut fair_supply = U256::max_value();

    for i in 0..amounts.len() {
        fair_supply = min(
            fair_supply,
            U256::from(amounts[i]) * U256::from(shares_total_supply) / total_amount[i],
        );
        println!("fair_supply: {}", fair_supply);
    }
    

    println!("test: {:?}", fair_supply.as_u128());

    println!("gas: {:?}", Gas(628361295471) / TGAS_DIVIDER);
    // println!("max_value: {:?}", U256::max_value().as_u128());
}
//        4800905168714284753
// minted 4623128061860842707
// 50000014684617098001319674162
