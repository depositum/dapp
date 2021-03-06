use crate::*;
use near_sdk::test_utils::*;
use near_sdk_sim::*;

static CONTRACT_STRATEGY: &[u8] = include_bytes!("../../../build/ref_farming_strategy.wasm");
const CONTRACT_ID: &str = "ref_frarming_strategy";
const TOKEN_ID: &str = "token.testnear";

fn init() -> (
    ContractAccount<RefFarmingStrategyContract>,
    Vec<UserAccount>,
) {
    let root = init_simulator(None);
    let contract = deploy!(
        contract: RefFarmingStrategyContract,
        contract_id: CONTRACT_ID.to_string(),
        bytes: CONTRACT_STRATEGY,
        signer_account: root,
        init_method: new(
            AccountId::new_unchecked("executor.near".to_string()),
            AccountId::new_unchecked("a.near".to_string()),
            AccountId::new_unchecked("b.near".to_string())
        ),
    );

    let mut user_list = vec![];
    for id in 0..2 {
        let user = root.create_user(accounts(id), to_yocto("200"));
        user_list.push(user);
    }

    // create second strategy
    (contract, user_list)
}

#[test]
#[ignore] // FIXME
fn farm() {
    let (contract, _user) = init();

    // initialized
    call!(contract.user_account, contract.init()).assert_success();

    // create first strategy
    let res = call!(
        contract.user_account,
        contract.supply(AccountId::new_unchecked(TOKEN_ID.to_string()), U128(100))
    );

    let strategy_id: U64 = res.unwrap_json();

    // check first id
    assert_eq!(strategy_id, U64::from(0));

    let res = call!(
        contract.user_account,
        contract.supply(
            AccountId::new_unchecked(TOKEN_ID.to_string()),
            U128::from(100)
        )
    );

    let strategy_id: U64 = res.unwrap_json();

    // check second id
    assert_eq!(strategy_id, U64::from(1));
}

#[test]
fn calc_swap_amount_out() {
    let (contract, _user) = init();

    // create first strategy
    // mut self, amount_in: u128, pool_info: &PoolInfo, slippage: u32
    let res = call!(
        contract.user_account,
        contract.calc_swap_amount_out(
            U128(100),
            PoolInfo {
                pool_kind: "Simple_pool".to_string(),
                /// List of tokens in the pool.
                token_account_ids: vec![],
                /// How much NEAR this contract has.
                amounts: vec![U128(100), U128(200)],
                /// Fee charged for swap.
                total_fee: 40,
                /// Total number of shares.
                shares_total_supply: U128(0),
            },
            10
        )
    );
    res.assert_success();
    // check first condition
    assert_eq!("99".to_string(), res.unwrap_json_value());
}
