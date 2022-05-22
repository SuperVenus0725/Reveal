use cosmwasm_std::{
    entry_point, to_binary,   CosmosMsg, Deps, DepsMut,Binary,Decimal,
    Env, MessageInfo, BankMsg, Response, StdResult, Uint128, WasmMsg, Coin, 
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, HopeMintMsg, InstantiateMsg, QueryMsg};
use crate::state::{
    State,CONFIG,UserInfo,MEMBERS
};
use cw721_base::{ExecuteMsg as Cw721BaseExecuteMsg, MintMsg};
use cw721::{Cw721ExecuteMsg};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        reveal_address: "reveal".to_string(),
        nft_address : "nft".to_string(),
        owner: info.sender.to_string(),
        denom: msg.denom,
        fee : msg.fee,
        royalty : msg.royalty,
        total_nft : Uint128::new(0),
        check_mint : msg.check_mint,
        can_mint : true
    };
    CONFIG.save(deps.storage,&state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::RevealNft { token_id,reveal_id, mint_msg } => execute_reveal_nft(deps, env, info, token_id,reveal_id,mint_msg),
        ExecuteMsg::SetAdminsList { members } => execute_set_members(deps,env,info,members),
        ExecuteMsg::SetRevealAddress { address } => execute_set_address(deps, info, address),
        ExecuteMsg::SetNftAddress { address } => execute_set_nft_address(deps, info, address),
        ExecuteMsg::RunMintFunction{flag} =>  execute_run_mint(deps,info,flag)
    }
}

fn execute_reveal_nft(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_id: String,
    reveal_id:i32,
    mint_msg:HopeMintMsg
) -> Result<Response, ContractError> {
   
    let state = CONFIG.load(deps.storage)?;
    let members = MEMBERS.load(deps.storage)?;

    let amount= info
        .funds
        .iter()
        .find(|c| c.denom == state.denom)
        .map(|c| Uint128::from(c.amount))
        .unwrap_or_else(Uint128::zero);

    if amount != state.fee{
        return Err(ContractError::Notenough {  });
    }

    if state.can_mint ==false {
        return Err(ContractError::CannotMint{})
    }

    CONFIG.update(deps.storage,
        | mut state|->StdResult<_>{
            state.total_nft = state.total_nft+Uint128::new(1);
            state.check_mint[(reveal_id-1) as usize]=false;
            Ok(state)
    })?;

    let mut messages:Vec<CosmosMsg> = vec![];

    for user in members{
        if user.portion == Decimal::zero(){
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: state.nft_address.clone(),
                funds:vec![],
                msg: to_binary(&Cw721ExecuteMsg::TransferNft { 
                    recipient: user.address,
                    token_id: token_id.clone() })?, 
               }))
        }
        else {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
                to_address: user.address,
                amount:vec![Coin{
                    denom:state.denom.clone(),
                    amount:amount*user.portion
                }]
        }))
    }
    }

    
    Ok(Response::new()
        .add_messages(messages)
        .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
             contract_addr: state.reveal_address, 
             msg: to_binary(&Cw721BaseExecuteMsg::Mint(MintMsg{
                token_id:["Reveal".to_string(),reveal_id.to_string()].join("."),
                owner:info.sender.to_string(),
                token_uri : mint_msg.clone().image,
                extension :mint_msg.clone()
             }))? , 
             funds: vec![] }))
)

}

fn execute_set_members(
    deps: DepsMut,
    _env:Env,
    info: MessageInfo,
    members: Vec<UserInfo>,
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;

    if info.sender.to_string() != state.owner{
        return Err(ContractError::Unauthorized {});
    }
    MEMBERS.save(deps.storage, &members)?;
    Ok(Response::default())
}


fn execute_run_mint(
    deps: DepsMut,
    info: MessageInfo,
    flag: bool,
)->Result<Response,ContractError>{

    let state = CONFIG.load(deps.storage)?;

    if info.sender.to_string() != state.owner{
        return Err(ContractError::Unauthorized {});
    }
    CONFIG.update(deps.storage,
    |mut state|->StdResult<_>{
        state.can_mint = flag;
        Ok(state)
    })?;
    Ok(Response::default())
}

fn execute_set_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
     let mut state = CONFIG.load(deps.storage)?;
    deps.api.addr_validate(&address)?;
    
    state.reveal_address = address;

    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(deps.storage, &state)?;
    Ok(Response::default())
}

fn execute_set_nft_address(
    deps: DepsMut,
    info: MessageInfo,
    address: String,
) -> Result<Response, ContractError> {
    let mut state = CONFIG.load(deps.storage)?;
    deps.api.addr_validate(&address)?;
    state.nft_address = address;
    
    if state.owner != info.sender.to_string() {
        return Err(ContractError::Unauthorized {});
    }

    CONFIG.save(deps.storage, &state)?;
    Ok(Response::default())
}


#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
          QueryMsg::GetStateInfo {  } => to_binary(&query_state_info(deps)?),
          QueryMsg::GetMembers {} => to_binary(&query_get_members(deps)?)
    }
}

pub fn query_state_info(deps:Deps) -> StdResult<State>{
    let state =  CONFIG.load(deps.storage)?;
    Ok(state)
}

pub fn query_get_members(deps:Deps) -> StdResult<Vec<UserInfo>>{
    let members = MEMBERS.load(deps.storage)?;
    Ok(members)
}


#[cfg(test)]
mod tests {

    use crate::msg::Trait;

    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{CosmosMsg};

    #[test]
    fn buy_token() {
        let mut deps = mock_dependencies();
        let instantiate_msg = InstantiateMsg {
            denom : "ujuno".to_string(),
            fee:Uint128::new(3000000),
            royalty : Decimal::from_ratio(5 as u128 , 100 as u128),
            check_mint : vec![true,true,true,true,true]
        };
        let info = mock_info("creator", &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, instantiate_msg).unwrap();
        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::SetNftAddress { address:"nft_address".to_string() };
        execute(deps.as_mut(),mock_env(),info,msg).unwrap();

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::SetRevealAddress { address:"reveal_address".to_string() };
        execute(deps.as_mut(),mock_env(),info,msg).unwrap();

        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state,State{
            reveal_address:"reveal_address".to_string(),
            nft_address : "nft_address".to_string(),
            owner:"creator".to_string(),
            denom : "ujuno".to_string(),
            fee : Uint128::new(3000000),
            royalty : Decimal::from_ratio(5 as u128,100 as u128),
            total_nft:Uint128::new(0),
            check_mint:vec![true,true,true,true,true],
            can_mint :  true
        });

        let info = mock_info("creator", &[]);
        let msg = ExecuteMsg::SetAdminsList { members:vec![
            UserInfo{
                address:"mint_pass".to_string(),
                portion:Decimal::from_ratio(0 as u128,100 as u128)
            },
            UserInfo{
                address:"user1".to_string(),
                portion:Decimal::from_ratio(1 as u128,100 as u128)
            }
            ,
            UserInfo{
                address:"user2".to_string(),
                portion:Decimal::from_ratio(2 as u128,100 as u128)
            }
        ] };
        execute(deps.as_mut(),mock_env(),info,msg).unwrap();
        let mint_msg= HopeMintMsg{
            name: Some("name".to_string()),
            description: Some("mint".to_string()),
            image: Some("image".to_string()),
            dna: Some("dna".to_string()),
            edition: Some(1),
            date: Some(mock_env().block.time.seconds()),
            attributes: Some(vec![Trait{
                trait_type:"type".to_string(),
                value:"value".to_string()
            }]),
            compiler : Some("compiler".to_string())
        
        };

        let info = mock_info("creator", &[Coin{
            denom:"ujuno".to_string(),
            amount:Uint128::new(3000000)
        }]);
        
        let msg = ExecuteMsg::RunMintFunction { flag:false};
         execute(deps.as_mut(),mock_env(),info,msg).unwrap();

         let info = mock_info("creator", &[Coin{
            denom:"ujuno".to_string(),
            amount:Uint128::new(3000000)
        }]);

        let msg = ExecuteMsg::RunMintFunction { flag:true};
         execute(deps.as_mut(),mock_env(),info,msg).unwrap();


        let info = mock_info("creator", &[Coin{
            denom:"ujuno".to_string(),
            amount:Uint128::new(3000000)
        }]);
        let msg = ExecuteMsg::RevealNft { token_id: "hope.1".to_string(),reveal_id:5, mint_msg:mint_msg.clone() };
        let res = execute(deps.as_mut(),mock_env(),info,msg).unwrap();
        assert_eq!(res.messages.len(),4);
        assert_eq!(res.messages[0].msg,CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: "nft_address".to_string(),
                funds:vec![],
                msg: to_binary(&Cw721ExecuteMsg::TransferNft { 
                    recipient: "mint_pass".to_string(),
                    token_id: "hope.1".to_string() }).unwrap(), 
               }));
        assert_eq!(res.messages[1].msg,CosmosMsg::Bank(BankMsg::Send {
                to_address: "user1".to_string(),
                amount:vec![Coin{
                    denom:"ujuno".to_string(),
                    amount:Uint128::new(30000)
                }]
        }));
         assert_eq!(res.messages[2].msg,CosmosMsg::Bank(BankMsg::Send {
                to_address: "user2".to_string(),
                amount:vec![Coin{
                    denom:"ujuno".to_string(),
                    amount:Uint128::new(60000)
                }]
        }));
        assert_eq!(res.messages[3].msg,CosmosMsg::Wasm(WasmMsg::Execute {
             contract_addr: state.reveal_address, 
             msg: to_binary(&Cw721BaseExecuteMsg::Mint(MintMsg{
                token_id:"Reveal.5".to_string(),
                owner:"creator".to_string(),
                token_uri : Some("image".to_string()),
                extension : mint_msg
             })).unwrap() , 
             funds: vec![] }));
        let state = query_state_info(deps.as_ref()).unwrap();
        assert_eq!(state.total_nft,Uint128::new(1));
         assert_eq!(state.check_mint,vec![true,true,true,true,false]);
    }
}
