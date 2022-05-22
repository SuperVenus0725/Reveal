use cosmwasm_std::{ Uint128, Decimal};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::state::UserInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom:String,
    pub fee : Uint128,
    pub royalty : Decimal,
    pub check_mint : Vec<bool>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    RevealNft{token_id:String,reveal_id:i32,mint_msg:HopeMintMsg},
    SetRevealAddress { address: String },
    SetNftAddress { address: String },
    SetAdminsList{members:Vec<UserInfo>},
    RunMintFunction{flag:bool}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
  GetStateInfo{},
  GetMembers{}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct HopeMintMsg {
    // Identifies the asset to which this NFT represents
    pub name: Option<String>,
    // A URI pointing to an image representing the asset
    pub description: Option<String>,
    // An external URI
    pub image: Option<String>,
    // Describes the asset to which this NFT represents (may be empty)
    pub dna: Option<String>,
    // royalties
    pub edition: Option<u64>,
    // initial ask price
    pub date: Option<u64>,
    // nft address of specified collection
    pub attributes: Option<Vec<Trait>>,

    pub compiler : Option<String>
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]

pub struct Trait{
    pub trait_type:String,
    pub value:String
}