#[test]
fn decrement() {
    let (root, contract, _alice) = init();

    let result = root.call(
        contract.account_id(),
        "decrement",
        &[].to_vec(),
        DEFAULT_GAS,
        0, // deposit
    );

    println!(
        "burnt tokens: {:.04}Ⓝ gas: {:.01} TeraGas",
        (result.tokens_burnt()) as f64 / 1e24,
        (result.gas_burnt().0) as f64 / 1e12,
    );

    assert!(result.gas_burnt() <= to_gas("2.7"));

    let actual: i8 = root
        .view(contract.account_id(), "get_num", &[].to_vec())
        .unwrap_json();

    assert_eq!(-1, actual);
}

#[test]
fn get_num() {
    let (root, contract, _alice) = init();

    let actual: i8 = root
        .view(contract.account_id(), "get_num", &[].to_vec())
        .unwrap_json();

    assert_eq!(0, actual);
}

#[test]
fn increment() {
    let (root, contract, _alice) = init();

    let result = root.call(
        contract.account_id(),
        "increment",
        &[].to_vec(),
        DEFAULT_GAS,
        0, // deposit
    );

    println!(
        "burnt tokens: {:.04}Ⓝ gas: {:.01} TeraGas",
        (result.tokens_burnt()) as f64 / 1e24,
        (result.gas_burnt().0) as f64 / 1e12,
    );

    assert!(result.gas_burnt() <= to_gas("2.7"));

    let actual: i8 = root
        .view(contract.account_id(), "get_num", &[].to_vec())
        .unwrap_json();

    assert_eq!(1, actual);
}

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    CONTRACT_COUNTER => "../../../build/counter.wasm",
}

use near_sdk::Gas;
use near_sdk_sim::init_simulator;
use near_sdk_sim::to_yocto;
use near_sdk_sim::UserAccount;
use near_sdk_sim::DEFAULT_GAS;
use near_sdk_sim::STORAGE_AMOUNT;

const CONTRACT_ID: &str = "contract";

pub fn init() -> (UserAccount, UserAccount, UserAccount) {
    // Use `None` for default genesis configuration; more info below
    let root = init_simulator(None);

    let contract = root.deploy(
        &CONTRACT_COUNTER,
        CONTRACT_ID.parse().unwrap(),
        STORAGE_AMOUNT, // attached deposit
    );

    let alice = root.create_user(
        "alice".parse().unwrap(),
        to_yocto("100"), // initial balance
    );

    (root, contract, alice)
}

pub fn to_gas(tera_gas: &str) -> Gas {
    let part: Vec<_> = tera_gas.split('.').collect();
    let number = part[0].parse::<u64>().unwrap() * u64::pow(10, 12);
    if part.len() > 1 {
        let power = part[1].len() as u32;
        let mantissa = part[1].parse::<u64>().unwrap() * u64::pow(10, 12 - power);
        Gas::from(number + mantissa)
    } else {
        Gas::from(number)
    }
}
