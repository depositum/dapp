use super::*;
use near_sdk::test_utils::accounts;
use near_sdk::test_utils::VMContextBuilder;
use near_sdk::testing_env;

// part of writing unit tests is setting up a mock context
fn context() -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder.signer_account_id(accounts(0));
    builder
}

// mark individual unit tests with #[test] for them to be registered and fired
#[test]
fn increment() {
    // set up the mock context into the testing environment
    testing_env!(context().build());
    // instantiate a contract variable with the counter at zero
    let mut contract = Contract::default();
    contract.increment();
    println!("Value after increment: {}", contract.get_num());
    // confirm that we received 1 when calling get_num
    assert_eq!(1, contract.get_num());
}

#[test]
fn decrement() {
    testing_env!(context().build());
    let mut contract = Contract::default();
    contract.decrement();
    println!("Value after decrement: {}", contract.get_num());
    // confirm that we received -1 when calling get_num
    assert_eq!(-1, contract.get_num());
}

#[test]
fn increment_and_reset() {
    testing_env!(context().build());
    let mut contract = Contract::default();
    contract.increment();
    contract.reset();
    println!("Value after reset: {}", contract.get_num());
    // confirm that we received -1 when calling get_num
    assert_eq!(0, contract.get_num());
}
