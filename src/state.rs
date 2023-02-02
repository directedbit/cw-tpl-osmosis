use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub count: i32,

    //owner of the contract, should only be able to modify it
    pub owner: Addr,
}

// State is just a massive key value storage

pub const STATE: Item<State> = Item::new("state");

pub const DEPOSITS: Map<&Addr, u128> = Map::new("deposits");

pub const DONATION_DENOM: Item<String> = Item::new("donation_denom");

//pub const DEPOSIT: Item<State> = Item::new(storage_key: "deposit");
