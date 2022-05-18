use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Wrong nft contract error")]
    WrongNftContract {},

    
    #[error("Not enough funds")]
    Notenough{},

    
 
    #[error("Escrow not expired")]
    NotExpired {},
}
