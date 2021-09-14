use crate::*;
use near_sdk::test_utils::*;
use near_sdk_sim::*;

lazy_static_include::lazy_static_include_bytes! {
    CONTRACT_REF_FARMING_STRATEGY => "../../../build/ref_farming_strategy.wasm",
}

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
        bytes: &CONTRACT_REF_FARMING_STRATEGY,
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
    (contract, user_list)
}

#[test]
fn farm() {
    let (contract, _user) = init();

    // create first strategy
    let res = call!(
        contract.user_account,
        contract.create(AccountId::new_unchecked(TOKEN_ID.to_string()), U128::from(100))
    );

    let strategy_id: U64 = res.unwrap_json();

    // check first id
    assert_eq!(strategy_id, U64::from(0));

    // create second strategy
    let res = call!(
        contract.user_account,
        contract.create(AccountId::new_unchecked(TOKEN_ID.to_string()), U128::from(100))
    );

    let strategy_id: U64 = res.unwrap_json();

    // check second id
    assert_eq!(strategy_id, U64::from(1));
}

