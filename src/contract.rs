use cosmwasm_std::{coins, BankMsg};
use std::u128;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{CountResponse, ExecuteMsg, InstantiateMsg, QueryMsg, SavingsBalanceResponse};
use crate::state::{State, DEPOSITS, DONATION_DENOM, STATE};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:osmo";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        count: msg.count,
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    STATE.save(deps.storage, &state)?;
    DONATION_DENOM.save(deps.storage, &msg.donation_denom)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender)
        .add_attribute("count", msg.count.to_string())
        .add_attribute("denom", msg.donation_denom))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Increment {} => try_increment(deps),
        ExecuteMsg::Reset { count } => try_reset(deps, info, count),
        ExecuteMsg::Donate {} => try_donate(deps, info, _env),
        ExecuteMsg::Deposit {} => try_deposit(deps, info, _env),
        ExecuteMsg::Withdraw { withdraw_amount } => try_withdraw(deps, info, _env, withdraw_amount),
    }
}
pub fn try_withdraw(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    withdraw_amount: u128,
) -> Result<Response, ContractError> {
    let denom = DONATION_DENOM.load(deps.storage)?;

    //let deposit_amount = cw_utils::must_pay(&info, &denom)?.u128();
    let current_depositor: bool = DEPOSITS.has(deps.storage, &info.sender);
    if !current_depositor {
        println!("welcome to the bank good ser but unfortunately you have no funds to withdraw");
        return Err(ContractError::Payment(cw_utils::PaymentError::NoFunds {}));
    }
    println!("hello good ser and welcome back to the bank.");
    let savings_balance = DEPOSITS.load(deps.storage, &info.sender)?;
    if withdraw_amount > savings_balance {
        println!("Unfortunately there's not that much money in your account");
        return Err(ContractError::Payment(cw_utils::PaymentError::NoFunds {}));
    }
    println!("No problems, we'll send that over.");
    let new_balance = savings_balance - withdraw_amount;
    DEPOSITS.save(deps.storage, &info.sender, &new_balance)?;
    let withdrawal_coins = coins(withdraw_amount, &denom);
    let message = BankMsg::Send {
        to_address: info.sender.into_string(),
        amount: withdrawal_coins,
    };
    let resp = Response::new()
        .add_message(message)
        .add_attribute("action", "withdraw")
        .add_attribute("amount", withdraw_amount.to_string());
    Ok(resp)
}
pub fn try_deposit(deps: DepsMut, info: MessageInfo, _env: Env) -> Result<Response, ContractError> {
    let denom = DONATION_DENOM.load(deps.storage)?;

    let deposit_amount = cw_utils::must_pay(&info, &denom)?.u128();
    let current_depositor: bool = DEPOSITS.has(deps.storage, &info.sender);
    if current_depositor {
        println!("hello good ser and welcome back to the bank.");
        let balance = DEPOSITS.load(deps.storage, &info.sender)?;
        let new_amount = balance + deposit_amount;
        DEPOSITS.save(deps.storage, &info.sender, &new_amount)?;
    } else {
        println!("welcome to the bank ser, it looks like you're new here");
        let new_amount = deposit_amount;
        DEPOSITS.save(deps.storage, &info.sender, &new_amount)?;
    }
    let resp: Response = Response::new()
        .add_attribute("action", "deposit")
        .add_attribute("amount", deposit_amount.to_string());
    Ok(resp)
}

pub fn try_donate(deps: DepsMut, info: MessageInfo, _env: Env) -> Result<Response, ContractError> {
    let denom = DONATION_DENOM.load(deps.storage)?;

    let donation = cw_utils::must_pay(&info, &denom)?.u128();

    /*let amnt = coins(donation, &denom);
    let myaddr = _env.contract.address;

    //dont need to send it to another account, it's been sent in
    let message = BankMsg::Send {
         to_address: myaddr.into_string(),
         amount: amnt,
    };*/
    let resp = Response::new()
        .add_attribute("action", "donate")
        .add_attribute("amount", donation.to_string())
        .add_attribute("total", "total");

    Ok(resp)
}

pub fn try_increment(deps: DepsMut) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        state.count += 1;
        Ok(state)
    })?;

    Ok(Response::new().add_attribute("method", "try_increment"))
}

pub fn try_reset(deps: DepsMut, info: MessageInfo, count: i32) -> Result<Response, ContractError> {
    STATE.update(deps.storage, |mut state| -> Result<_, ContractError> {
        if info.sender != state.owner {
            return Err(ContractError::Unauthorized {});
        }
        state.count = count;
        Ok(state)
    })?;
    Ok(Response::new().add_attribute("method", "reset"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetSavingsBalance { depositor } => {
            to_binary(&query_savings_balance(deps, depositor)?)
        }
    }
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let state = STATE.load(deps.storage)?;
    Ok(CountResponse { count: state.count })
}

fn query_savings_balance(deps: Deps, depositor: Addr) -> StdResult<SavingsBalanceResponse> {
    let deposit = DEPOSITS.load(deps.storage, &depositor)?;
    Ok(SavingsBalanceResponse {
        savings_balance: deposit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies_with_balance, mock_env, mock_info};
    use cosmwasm_std::{coin, coins, from_binary, Addr};
    use cw_multi_test::ContractWrapper;
    use cw_multi_test::{App, Executor};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {
            count: 17,
            donation_denom: "usd".to_string(),
        };
        let info = mock_info("creator", &coins(1000, "earth"));

        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        // it worked, let's query the state
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.count);
    }

    #[test]
    fn deposit() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "fauxcoin"));

        let msg = InstantiateMsg {
            count: 17,
            donation_denom: "fauxcoin".to_string(),
        };
        let info = mock_info("creator", &coins(2, "fauxcoin"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // anyone can deposit
        let info = mock_info("anyone", &coins(7, "fauxcoin"));
        let msg = ExecuteMsg::Deposit {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should balance by amount
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetSavingsBalance {
                depositor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();

        let value: SavingsBalanceResponse = from_binary(&res).unwrap();
        assert_eq!(7, value.savings_balance);

        let second_info = mock_info("anyone", &coins(4, "fauxcoin"));
        let second_msg = ExecuteMsg::Deposit {};
        let _second_res: Response =
            execute(deps.as_mut(), mock_env(), second_info, second_msg).unwrap();

        // should balance by amount
        let second_res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetSavingsBalance {
                depositor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();

        let value: SavingsBalanceResponse = from_binary(&second_res).unwrap();
        assert_eq!(11, value.savings_balance);
    }

    #[test]
    fn withdraw() {
        let mut deps = mock_dependencies_with_balance(&coins(6, "fauxcoin"));

        let msg = InstantiateMsg {
            count: 17,
            donation_denom: "fauxcoin".to_string(),
        };
        let info = mock_info("creator", &coins(2, "fauxcoin"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // anyone can deposit
        let info = mock_info("anyone", &coins(7, "fauxcoin"));
        let msg = ExecuteMsg::Deposit {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should balance by amount
        let res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetSavingsBalance {
                depositor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();

        let value: SavingsBalanceResponse = from_binary(&res).unwrap();
        assert_eq!(7, value.savings_balance);

        let second_info = mock_info("anyone", &[]);
        let second_msg = ExecuteMsg::Withdraw { withdraw_amount: 3 };
        let _second_res: Response =
            execute(deps.as_mut(), mock_env(), second_info, second_msg).unwrap();

        // should balance by amount
        let second_res = query(
            deps.as_ref(),
            mock_env(),
            QueryMsg::GetSavingsBalance {
                depositor: Addr::unchecked("anyone"),
            },
        )
        .unwrap();

        let value: SavingsBalanceResponse = from_binary(&second_res).unwrap();
        assert_eq!(4, value.savings_balance);
    }

    #[test]
    fn increment() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {
            count: 17,
            donation_denom: "fakeusd".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Increment {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        // should increase counter by 1
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(18, value.count);
    }

    #[test]
    fn reset() {
        let mut deps = mock_dependencies_with_balance(&coins(2, "token"));

        let msg = InstantiateMsg {
            count: 17,
            donation_denom: "fakeusd".to_string(),
        };
        let info = mock_info("creator", &coins(2, "token"));
        let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        // beneficiary can release it
        let unauth_info = mock_info("anyone", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let res = execute(deps.as_mut(), mock_env(), unauth_info, msg);
        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("Must return unauthorized error"),
        }

        // only the original creator can reset the counter
        let auth_info = mock_info("creator", &coins(2, "token"));
        let msg = ExecuteMsg::Reset { count: 5 };
        let _res = execute(deps.as_mut(), mock_env(), auth_info, msg).unwrap();

        // should now be 5
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetCount {}).unwrap();
        let value: CountResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.count);
    }

    #[test]
    #[should_panic(expected = "Must send reserve token")]
    fn donate_wrong_denom() {
        let init_funds = vec![coin(5, "fauxcoin"), coin(5, "eth")];
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), init_funds)
                .unwrap()
        });
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    donation_denom: "eth".to_string(),
                    count: 100,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();
        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(5, "fauxcoin"),
        )
        .unwrap();
    }

    #[test]
    fn donations() {
        let init_funds = vec![coin(5, "fauxcoin"), coin(5, "eth")];
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked("user"), init_funds)
                .unwrap()
        });
        let code = ContractWrapper::new(execute, instantiate, query);
        let code_id = app.store_code(Box::new(code));
        let addr = app
            .instantiate_contract(
                code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    donation_denom: "eth".to_string(),
                    count: 100,
                },
                &[],
                "Contract",
                None,
            )
            .unwrap();
        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(5, "eth"),
        )
        .unwrap();
        /*app.execute_contract(
            Addr::unchecked("owner"),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(5, "eth"),
        )
        .unwrap();*/
        //user has 0
        assert_eq!(
            app.wrap()
                .query_balance("user", "eth")
                .unwrap()
                .amount
                .u128(),
            0
        );

        //contract has 5
        assert_eq!(
            app.wrap()
                .query_balance(&addr, "eth")
                .unwrap()
                .amount
                .u128(),
            5
        );

        app.execute_contract(
            Addr::unchecked("user"),
            addr.clone(),
            &ExecuteMsg::Donate {},
            &coins(5, "fauxcoin"),
        )
        .unwrap();
    }
}
