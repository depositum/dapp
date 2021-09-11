
use near_sdk::collections::LookupMap;
use near_sdk::near_bindgen;
use near_sdk::PanicOnDefault;
use near_sdk::borsh::BorshDeserialize;
use near_sdk::borsh::BorshSerialize;
use near_sdk::borsh;

#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct Contract {
}


#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self { }
    }

    pub fn coin_list(&self) -> Vec<String> {
        vec![]
    }
}

