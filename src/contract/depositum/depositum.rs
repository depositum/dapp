use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::assert_self;
use near_sdk::borsh;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::borsh::BorshSerialize;
use near_sdk::collections::UnorderedMap;
use near_sdk::collections::{LookupMap, UnorderedSet};
use near_sdk::env;
use near_sdk::ext_contract;
use near_sdk::is_promise_success;
use near_sdk::json_types::U128;
use near_sdk::log;
use near_sdk::near_bindgen;
use near_sdk::require;
use near_sdk::serde::Serialize;
use near_sdk::serde_json::json;
use near_sdk::AccountId;
use near_sdk::Balance;
use near_sdk::BorshStorageKey;
use near_sdk::Gas;
use near_sdk::PanicOnDefault;
use near_sdk::Promise;
use near_sdk::PromiseOrValue;
use near_sdk::PromiseResult;

#[cfg(all(test, not(target_arch = "wasm32")))]
mod simulator;

const GAS_PROMISE_CALL: Gas = Gas(5_000_000_000_000);
const DEPOSIT_COIN_REGISTER: Balance = 1_250_000_000_000_000_000_000;

const NO_DEPOSIT: Balance = 0;

pub const TGAS: u64 = 1_000_000_000_000;
pub const BASE_GAS: Gas = Gas(25 * TGAS);
pub const LOCKUP_NEW: Gas = BASE_GAS;
pub const CALLBACK: Gas = BASE_GAS;
const RESERVE_TGAS: Gas = Gas(10_000_000_000_000);
const FT_TRANSFER_CALL_GAS: Gas = Gas(50_000_000_000_000);

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Coin,
    Deposit,
    CoinBalance { beneficiary: AccountId },
    StrategyDeposit,
}

type CoinAccountId = AccountId;

type Deposit = UnorderedMap<CoinAccountId, Balance>;

const STRATEGY_CODE: &[u8] = include_bytes!("res/ref_farming_strategy.wasm");

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Depositum {
    coin: UnorderedSet<AccountId>,
    account: LookupMap<AccountId, Deposit>,
    strategy_deposit: LookupMap<AccountId, U128>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct StrategyArgs {
    executor: AccountId,
    ref_farming_account: AccountId,
    ref_exchange_account: AccountId,
}

#[ext_contract(ext_self)]
pub trait ExtSelf {
    fn on_lockup_create(
        &mut self,
        lockup_account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> bool;
    fn callback_ft_transfer_call(
        &mut self,
        sub_account_id: AccountId,
        account_id: AccountId,
        amount: U128,
    );
    fn callback_after_withdraw(&mut self, account_id: AccountId, coin: AccountId, amount: U128);
}

#[ext_contract(ft_token)]
pub trait FtToken {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
    fn ft_transfer(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(strategy_contract)]
pub trait StrategyContract {
    fn supply(&mut self, token: AccountId, amount: U128) -> PromiseOrValue<U64>;
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
            strategy_deposit: LookupMap::new(StorageKey::StrategyDeposit),
        };
        this.coin.insert(&"near".parse().unwrap());
        this
    }

    pub fn coin_list(&self) -> Vec<AccountId> {
        self.coin.to_vec()
    }

    // #[private]
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

    // #[private]
    pub fn coin_disable(&mut self, coin: AccountId) {
        self.coin.remove(&coin);
        log!("coin {} disabled", coin);
    }

    pub fn strategy_list(&self) -> Vec<String> {
        vec![]
    }

    fn deposit_subtract(
        &mut self,
        coin: &AccountId,
        beneficiary: &AccountId,
        amount: U128,
    ) -> U128 {
        require!(amount.0 > 0, "Empty amount");
        log!("deposit_update");
        let mut account = self
            .account
            .get(&beneficiary.clone())
            .expect("account not found");

        let deposit = match account.get(coin) {
            None => amount.0,
            Some(deposit) => deposit.checked_sub(amount.0).expect("Error update deposit"),
        };
        account.insert(coin, &deposit);
        self.account.insert(&beneficiary.clone(), &account);
        log!("deposit updated {} for {}", deposit, beneficiary,);
        U128(deposit)
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

    pub fn get_strategy_deposit(&self, sub_account_id: AccountId) -> Option<U128> {
        self.strategy_deposit.get(&sub_account_id)
    }

    pub fn withdrawal(&self, coin: &AccountId, amount: U128) {
        log!("withdrawal {} {:?}", coin, amount);
        let account = self
            .account
            .get(&AccountId::new_unchecked(
                env::predecessor_account_id().to_string(),
            ))
            .expect("account not found");

        let coin_balance = account.get(coin).unwrap_or(0);

        if coin_balance >= amount.0 {
            let gas_for_next_callback =
                env::prepaid_gas() - env::used_gas() - FT_TRANSFER_CALL_GAS - RESERVE_TGAS;

            ft_token::ft_transfer(
                env::predecessor_account_id(),
                amount,
                "".to_string(),
                coin.clone(),
                1, // todo create constant
                FT_TRANSFER_CALL_GAS,
            )
            .then(ext_self::callback_after_withdraw(
                env::predecessor_account_id(),
                coin.clone(),
                amount,
                env::current_account_id(),
                0,
                gas_for_next_callback,
            ));
        } else {
            log!(
                "Account {} balance is {}, but requested {:?} to withdraw",
                env::predecessor_account_id(),
                coin_balance,
                amount
            );
        }
    }

    #[private]
    pub fn callback_after_withdraw(
        &mut self,
        account_id: AccountId,
        coin: AccountId,
        amount: U128,
    ) {
        let is_success = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => true,
            PromiseResult::Failed => false,
        };

        if is_success {
            self.deposit_subtract(&coin, &account_id, amount);
        }
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

    #[payable]
    pub fn create(&mut self, accound_sub_id: String) -> Promise {
        // self.assert_custodian();
        let amount = env::attached_deposit();

        let lockup_account_id = format!(
            "{}.{}",
            // hex::encode(&byte_slice[..20]),
            accound_sub_id,
            env::current_account_id(),
        );

        Promise::new(AccountId::new_unchecked(lockup_account_id.clone()))
            .create_account()
            .deploy_contract(STRATEGY_CODE.to_vec())
            .transfer(amount)
            .function_call(
                "new".to_string(),
                near_sdk::serde_json::to_vec(&StrategyArgs {
                    executor: env::predecessor_account_id(),
                    ref_farming_account: AccountId::new_unchecked(
                        "ref-farming-aromankov.testnet".to_string(),
                    ),
                    ref_exchange_account: AccountId::new_unchecked(
                        "ref-exchange-aromankov.testnet".to_string(),
                    ),
                })
                .unwrap(),
                NO_DEPOSIT,
                LOCKUP_NEW,
            )
            .then(ext_self::on_lockup_create(
                AccountId::new_unchecked(lockup_account_id.clone()),
                env::attached_deposit().into(),
                env::predecessor_account_id(),
                env::current_account_id(),
                NO_DEPOSIT,
                CALLBACK,
            ))
    }

    pub fn on_delete(&mut self, accound_sub_id: AccountId) {
        self.strategy_deposit
            .insert(&accound_sub_id.clone(), &U128(0));
    }

    #[payable]
    pub fn start_strategy(
        &mut self,
        account_id: AccountId,
        sub_account_id: AccountId,
        amount: U128,
    ) -> Promise {
        self.strategy_deposit
            .insert(&sub_account_id.clone(), &amount.clone());

        let gas_for_next_callback = env::prepaid_gas()
            - env::used_gas()
            - FT_TRANSFER_CALL_GAS
            - RESERVE_TGAS
            - Gas(2_000_000_000_000);

        log!(
            "start_strategy, gas_for_next_callback: {:?}",
            gas_for_next_callback
        );
        log!("start_strategy, used_gas {:?}", env::used_gas());
        log!("start_strategy, prepaid_gas {:?}", env::prepaid_gas());

        ft_token::ft_transfer_call(
            sub_account_id.clone(),
            amount,
            "".to_string(),
            AccountId::new_unchecked("wrap_near-aromankov.testnet".to_string()),
            1, // todo create constant
            FT_TRANSFER_CALL_GAS,
        )
        .then(ext_self::callback_ft_transfer_call(
            sub_account_id.clone(),
            account_id.clone(),
            amount,
            env::current_account_id(),
            0,
            gas_for_next_callback,
        ))
    }

    #[private]
    pub fn callback_ft_transfer_call(
        &mut self,
        sub_account_id: AccountId,
        account_id: AccountId,
        amount: U128,
    ) {
        log!(
            "callback_ft_transfer_call, prepaid_gas {:?}",
            env::prepaid_gas()
        );
        let is_success = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_value) => true,
            PromiseResult::Failed => false,
        };

        if is_success {
            let gas_for_next_callback = env::prepaid_gas() - env::used_gas() - RESERVE_TGAS;

            log!(
                "callback_ft_transfer_call, gas_for_next_callback: {:?}",
                gas_for_next_callback
            );
            log!("callback_ft_transfer_call, used_gas {:?}", env::used_gas());

            let coin = AccountId::new_unchecked("wrap_near-aromankov.testnet".to_string());

            self.deposit_subtract(&coin, &account_id, amount);

            // call supply of sub_account_id
            strategy_contract::supply(
                coin,
                amount,
                sub_account_id,
                NO_DEPOSIT,
                gas_for_next_callback,
            );
        } else {
            log!("ft_transfer_call not successfull");
        }
    }

    pub fn on_lockup_create(
        &mut self,
        lockup_account_id: AccountId,
        attached_deposit: U128,
        predecessor_account_id: AccountId,
    ) -> bool {
        assert_self();

        let lockup_account_created = is_promise_success();

        if lockup_account_created {
            log!(
                "The lockup contract {} was successfully created.",
                lockup_account_id
            );
            true
        } else {
            log!(
                "The lockup {} creation has failed. Returning attached deposit of {} to {}",
                lockup_account_id,
                attached_deposit.0,
                predecessor_account_id
            );

            Promise::new(predecessor_account_id).transfer(attached_deposit.0);
            false
        }
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
        let receiver: AccountId = if msg.len() > 0 {
            let sub_acc_id = AccountId::new_unchecked(msg.clone());

            self.strategy_deposit.insert(&sub_acc_id.clone(), &U128(0));

            sub_acc_id
        } else {
            sender_id.clone()
        };
        self.deposit_update(&coin, &receiver, amount, msg);
        PromiseOrValue::Value(U128(0))
    }
}
