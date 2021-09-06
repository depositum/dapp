use crate::*;
use near_sdk::test_utils::*;
use near_sdk_sim::*;
use simple_token::*;

lazy_static_include::lazy_static_include_bytes! {
    CONTRACT_SIMPLE_TOKEN => "../../../build/simple_token.wasm",
    CONTRACT_DEPOSITUM => "../../../build/depositum.wasm",
}

const CONTRACT_ID: &str = "depositum";
const COIN_ID: &str = "coin";

fn init() -> (
    ContractAccount<DepositumContract>,
    ContractAccount<SimpleTokenContract>,
    Vec<UserAccount>,
) {
    let root = init_simulator(None);
    let contract = deploy!(
        contract: DepositumContract,
        contract_id: CONTRACT_ID.to_string(),
        bytes: &CONTRACT_DEPOSITUM,
        signer_account: root,
        init_method: new()
    );
    let coin = deploy!(
        contract: SimpleTokenContract,
        contract_id: COIN_ID.to_string(),
        bytes: &CONTRACT_SIMPLE_TOKEN,
        signer_account: root,
        init_method: new("coin".to_string(), None)
    );
    call!(
        contract.user_account,
        contract.coin_enable(AccountId::new_unchecked(COIN_ID.to_string()))
    );

    let mut user_list = vec![];
    for id in 0..2 {
        let user = root.create_user(accounts(id), to_yocto("200"));
        call!(user, coin.ft_mint(), deposit = to_yocto("100.00125"));
        user_list.push(user);
    }
    (contract, coin, user_list)
}

#[test]
fn deposit() {
    let (contract, coin, user) = init();
    let actual: U128 = view!(coin.ft_balance_of(accounts(0))).unwrap_json();
    assert_eq!(100000000000000000000000000, actual.0);
    let out = call!(
        user[0],
        coin.ft_transfer_call(
            AccountId::new_unchecked(CONTRACT_ID.to_string()),
            U128(to_yocto("50")),
            None,
            "".to_string()
        ),
        deposit = 1
    );
    out.assert_success();
    let actual: U128 = view!(coin.ft_balance_of(accounts(0))).unwrap_json();
    assert_eq!(50000000000000000000000000, actual.0);
    assert_eq!(*out.logs(), vec!["ft_resolve_transfer".to_string()]);
    let actual: Vec<(CoinAccountId, U128)> = view!(contract.balance_of(accounts(0))).unwrap_json();
    assert_eq!(actual.len(), 1);
    assert_eq!(
        (
            AccountId::new_unchecked(COIN_ID.to_string()),
            U128(to_yocto("50"))
        ),
        actual[0]
    );
}

#[test]
fn coin_list() {
    let (contract, _, _) = init();
    let actual: Vec<AccountId> = view!(contract.coin_list()).unwrap_json();
    assert_eq!(
        vec![
            AccountId::new_unchecked("near".to_string()),
            AccountId::new_unchecked("coin".to_string()),
        ],
        actual
    );
}
