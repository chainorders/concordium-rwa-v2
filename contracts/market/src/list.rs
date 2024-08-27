use concordium_rwa_utils::state_implementations::token_deposits_state::IDepositedTokensState;
use concordium_std::*;

use super::error::Error;
use super::event::{Event, TokenDeListed, TokenListed};
use super::state::State;
use super::types::{Cis2TokenAmount, ContractResult, ExchangeRate, TokenOwnerUId, TokenUId};

#[derive(Serialize, SchemaType)]
pub struct ListParams {
    pub token_id:       TokenUId,
    pub owner:          AccountAddress,
    pub exchange_rates: Vec<ExchangeRate>,
    pub supply:         Cis2TokenAmount,
}

#[receive(
    contract = "rwa_market",
    name = "list",
    enable_logger,
    mutable,
    parameter = "ListParams",
    error = "super::error::Error"
)]
pub fn list(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: ListParams = ctx.parameter_cursor().get()?;
    ensure!(
        ctx.sender().matches_account(&params.owner),
        Error::Unauthorized
    );

    list_internal(params, host, logger)
}

pub fn list_internal(
    params: ListParams,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(params.supply.ge(&0u64.into()), Error::InvalidSupply);
    ensure!(
        params.exchange_rates.len().ge(&1),
        Error::InvalidExchangeRates
    );
    ensure!(
        params.exchange_rates.len().le(&u8::MAX.into()),
        Error::InvalidExchangeRates
    );

    let state = host.state();
    ensure!(state.can_list(&params.token_id), Error::InvalidListToken);
    ensure!(
        params.exchange_rates.iter().all(|rate| {
            match rate {
                ExchangeRate::Ccd(rate) => rate.is_valid(),
                ExchangeRate::Cis2((token_uid, rate)) => {
                    rate.is_valid() && state.can_be_paid_by(token_uid)
                }
            }
        }),
        Error::InvalidExchangeRates
    );
    // Checking deposited amount because listing of a token is replaced by a new
    // listing request
    let token_id = TokenOwnerUId::new(params.token_id.to_owned(), params.owner.into());
    ensure!(
        state.balance_of_deposited(&token_id).ge(&params.supply),
        Error::InsufficientDeposits
    );
    host.state_mut()
        .add_or_replace_listed(token_id, params.supply, params.exchange_rates)?;
    logger.log(&Event::Listed(TokenListed {
        token_id: params.token_id,
        owner:    params.owner,
        supply:   params.supply,
    }))?;

    Ok(())
}

#[derive(Serialize, SchemaType)]
pub struct GetListedParam {
    pub token_id: TokenUId,
    pub owner:    AccountAddress,
}

#[derive(Serialize, SchemaType)]
pub struct ListedToken {
    pub token_id:       TokenUId,
    pub owner:          AccountAddress,
    pub exchange_rates: Vec<ExchangeRate>,
    pub supply:         Cis2TokenAmount,
}

#[receive(
    contract = "rwa_market",
    name = "getListed",
    parameter = "GetListedParam",
    return_value = "ListedToken",
    error = "super::error::Error"
)]
pub fn get_listed(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<ListedToken> {
    let params: GetListedParam = ctx.parameter_cursor().get()?;
    let state = host.state();
    let listed_token = state
        .get_listed(&TokenOwnerUId::new(
            params.token_id.to_owned(),
            params.owner.into(),
        ))
        .map(|(supply, exchange_rates)| ListedToken {
            token_id: params.token_id,
            owner: params.owner,
            exchange_rates,
            supply,
        })
        .ok_or(Error::NotListed)?;

    Ok(listed_token)
}

#[receive(
    contract = "rwa_market",
    name = "balanceOfListed",
    parameter = "GetListedParam",
    return_value = "Cis2TokenAmount",
    error = "super::error::Error"
)]
pub fn balance_of_listed(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<Cis2TokenAmount> {
    let params: GetListedParam = ctx.parameter_cursor().get()?;
    let state = host.state();
    let balance = state.listed_amount(&TokenOwnerUId {
        token_id: params.token_id,
        owner:    params.owner.into(),
    });

    Ok(balance)
}

#[receive(
    contract = "rwa_market",
    name = "balanceOfUnlisted",
    parameter = "GetListedParam",
    return_value = "Cis2TokenAmount",
    error = "super::error::Error"
)]
pub fn balance_of_unlisted(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<Cis2TokenAmount> {
    let params: GetListedParam = ctx.parameter_cursor().get()?;
    let state = host.state();
    let balance = state.unlisted_amount(&TokenOwnerUId::new(params.token_id, params.owner.into()));

    Ok(balance)
}

#[derive(Serialize, SchemaType)]
pub struct DeListParams {
    pub token_id: TokenUId,
    pub owner:    AccountAddress,
}

#[receive(
    contract = "rwa_market",
    name = "deList",
    enable_logger,
    mutable,
    parameter = "DeListParams",
    error = "super::error::Error"
)]
pub fn de_list(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: DeListParams = ctx.parameter_cursor().get()?;
    ensure!(
        ctx.sender().matches_account(&params.owner),
        Error::Unauthorized
    );
    let token_id = TokenOwnerUId::new(params.token_id, params.owner.into());

    let state = host.state_mut();
    state.remove_listed(&token_id)?;
    logger.log(&Event::DeListed(TokenDeListed {
        token_id: token_id.token_id,
        owner:    params.owner,
    }))?;

    Ok(())
}
