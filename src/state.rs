use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128,Decimal};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, ReadonlySingleton, Singleton,
};
use cw_storage_plus::{Map,Item};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


pub const CONFIG: Item<State> = Item::new("config_state");
pub const MEMBERS : Item<Vec<UserInfo>> = Item::new("config_members");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub reveal_address:String,
    pub nft_address : String,
    pub owner:String,
    pub denom : String,
    pub fee : Uint128,
    pub royalty : Decimal,
    pub total_nft:Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserInfo {
    pub address: String,
    pub portion:Decimal
}