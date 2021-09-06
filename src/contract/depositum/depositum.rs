use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::borsh;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::UnorderedMap;
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::env;
use near_sdk::json_types::U128;
use near_sdk::log;
use near_sdk::near_bindgen;
use near_sdk::require;
use near_sdk::serde_json::json;
use near_sdk::AccountId;
use near_sdk::Balance;
use near_sdk::BorshStorageKey;
use near_sdk::Gas;
use near_sdk::PanicOnDefault;
use near_sdk::PromiseOrValue;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod simulator;

const GAS_PROMISE_CALL: Gas = Gas(5_000_000_000_000);
const GAS_FOR_RESOLVE_TRANSFER: Gas = GAS_PROMISE_CALL;
const GAS_FOR_FT_TRANSFER_CALL: Gas = Gas(25_000_000_000_000 + GAS_FOR_RESOLVE_TRANSFER.0);
const DEPOSIT_COIN_REGISTER: Balance = 1_250_000_000_000_000_000_000;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Coin,
    Deposit,
    CoinBalance { beneficiary: AccountId },
}

type CoinAccountId = AccountId;

type Deposit = UnorderedMap<CoinAccountId, Balance>;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Depositum {
    coin: UnorderedSet<AccountId>,
    account: LookupMap<AccountId, Deposit>,
}

macro_rules! arguments {
    ($($json:tt)+) => {
        json!($($json)+).to_string().as_bytes()
    };
}

#[near_bindgen]
impl Depositum {
    #[init]
    pub fn new() -> Self {
        let mut this = Self {
            coin: UnorderedSet::new(StorageKey::Coin),
            account: LookupMap::new(StorageKey::Deposit),
        };
        this.coin.insert(&"near".parse().unwrap());
        this
    }

    pub fn coin_list(&self) -> Vec<AccountId> {
        self.coin.to_vec()
    }

    #[private]
    pub fn coin_enable(&mut self, coin: AccountId) {
        let promise = env::promise_create(
            coin.clone(),
            "storage_deposit",
            arguments!({
                "account_id": env::current_account_id(),
            }),
            DEPOSIT_COIN_REGISTER,
            GAS_PROMISE_CALL,
        );
        env::promise_return(promise);
        self.coin.insert(&coin);
        log!("coin {} enabled", coin);
    }

    #[private]
    pub fn coin_disable(&mut self, coin: AccountId) {
        self.coin.remove(&coin);
        log!("coin {} disabled", coin);
    }

    pub fn strategy_list(&self) -> Vec<String> {
        vec![]
    }

    fn deposit_update(
        &mut self,
        coin: &AccountId,
        beneficiary: &AccountId,
        amount: U128,
        msg: String,
    ) -> U128 {
        require!(amount.0 > 0, "Empty amount");
        log!("deposit_update");
        let mut account = match self.account.get(&beneficiary.clone()) {
            None => {
                let deposit = Deposit::new(StorageKey::CoinBalance {
                    beneficiary: beneficiary.clone(),
                });
                self.account.insert(&beneficiary.clone(), &deposit);
                deposit
            }
            Some(deposit) => deposit,
        };
        let deposit = match account.get(coin) {
            None => amount.0,
            Some(deposit) => deposit.checked_add(amount.0).expect("Error update deposit"),
        };
        account.insert(coin, &deposit);
        self.account.insert(&beneficiary.clone(), &account);
        log!(
            "deposit updated {} for {} with {}",
            deposit,
            beneficiary,
            msg
        );
        U128(deposit)
    }

    pub fn balance_of(&self, account_id: AccountId) -> Vec<(CoinAccountId, U128)> {
        match self.account.get(&account_id) {
            None => vec![],
            Some(deposit) => deposit.iter().map(|x| (x.0, U128(x.1))).collect(),
        }
    }

    pub fn withdrawal(&self) {
        log!("withdrawal")
    }

    pub fn withdrawal_force(&self) {
        log!("withdrawal_force")
    }

    pub fn decision_do(&self) {
        log!("decision_do")
    }

    fn coin_enable_require(&self, coin: &AccountId) {
        require!(self.coin.contains(coin), "Coin not support")
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Depositum {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!("ft_on_transfer");
        let coin = env::predecessor_account_id();
        self.coin_enable_require(&coin);
        //let amount: Balance = amount.into();
        self.deposit_update(&coin, &sender_id, amount, msg);
        PromiseOrValue::Value(U128(0))
    }
}
