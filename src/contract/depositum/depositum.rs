use near_sdk::assert_one_yocto;
use near_sdk::borsh;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::LookupMap;
use near_sdk::collections::UnorderedMap;
use near_sdk::env;
use near_sdk::json_types::U128;
use near_sdk::near_bindgen;
use near_sdk::require;
use near_sdk::AccountId;
use near_sdk::Balance;
use near_sdk::BorshStorageKey;
use near_sdk::PanicOnDefault;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Deposit,
    CoinBalance { beneficiary_id: AccountId },
}

type CoinAccountId = AccountId;

type Deposit = UnorderedMap<CoinAccountId, Balance>;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
    deposit: LookupMap<AccountId, Deposit>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            deposit: LookupMap::new(StorageKey::Deposit),
        }
    }

    pub fn coin_list(&self) -> Vec<String> {
        vec![]
    }

    pub fn coin_enable(&mut self, coin: AccountId) {
        env::log_str(&format!("coin {} enabled", coin))
    }

    pub fn coin_disable(&mut self, coin: AccountId) {
        env::log_str(&format!("coin {} disabled", coin));
    }

    pub fn strategy_list(&self) -> Vec<String> {
        vec![]
    }

    pub fn deposit_list(&self) -> Vec<String> {
        vec![]
    }

    #[payable]
    pub fn deposit(&mut self, coin: AccountId, amount: U128) -> (AccountId, U128) {
        assert_one_yocto();
        ("usd".parse().unwrap(), U128(0))
    }

    pub fn balance_of(&self, account_id: AccountId) -> Vec<(CoinAccountId, U128)> {
        match self.deposit.get(&account_id) {
            None => vec![],
            Some(_) => vec![],
        }
    }

    pub fn withdrawal(&self) {
        env::log_str("withdrawal")
    }

    pub fn withdrawal_force(&self) {
        env::log_str("withdrawal_force")
    }

    pub fn decision_do(&self) {
        env::log_str("decision_do")
    }
}
