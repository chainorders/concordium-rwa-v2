use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_protocols::concordium_cis2_security::{FreezeParam, TokenFrozen};
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::*;

/// Freezes the given amount of given tokenIds for the given address.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was successful.
///
/// # Errors
///
/// Returns `Error::Unauthorized` if the sender is not an agent.
/// Returns `Error::InsufficientFunds` if the owner does not have enough unfrozen balance.
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_rewards",
    name = "freeze",
    mutable,
    enable_logger,
    parameter = "FreezeParams",
    error = "Error"
)]
pub fn freeze(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let is_authorized = host
        .state()
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::Freeze]));
    ensure!(is_authorized, Error::Unauthorized);

    let FreezeParams {
        owner: owner_address,
        tokens: freezes,
    }: FreezeParams = ctx.parameter_cursor().get()?;

    let state = host.state();
    for freeze in &freezes {
        state
            .token(&freeze.token_id)
            .ok_or(Error::InvalidTokenId)?
            .main()
            .ok_or(Error::InvalidTokenId)?;
    }

    let state = host.state_mut();
    let mut owner = state
        .address_mut(&owner_address)
        .ok_or(Error::InvalidAddress)?;
    let owner = owner.holder_mut().ok_or(Error::InvalidAddress)?;
    let owner = owner.active_mut().ok_or(Error::RecoveredAddress)?;

    for FreezeParam {
        token_id,
        token_amount,
    } in freezes
    {
        ensure!(token_amount.gt(&TokenAmount::zero()), Error::InvalidAmount);
        owner
            .balance_mut(&token_id)
            .ok_or(Error::InsufficientFunds)?
            .freeze(token_amount)?;
        logger.log(&Event::TokenFrozen(TokenFrozen {
            token_id,
            amount: token_amount,
            address: owner_address,
        }))?
    }

    Ok(())
}

/// Unfreezes the given amount of given tokenIds for the given address.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was successful.
///
/// # Errors
///
/// Returns `Error::Unauthorized` if the sender is not an agent.
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_rewards",
    name = "unFreeze",
    mutable,
    enable_logger,
    parameter = "FreezeParams",
    error = "Error"
)]
pub fn un_freeze(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let is_authorized = host
        .state()
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::Freeze]));
    ensure!(is_authorized, Error::Unauthorized);

    let FreezeParams {
        owner: owner_address,
        tokens: freezes,
    }: FreezeParams = ctx.parameter_cursor().get()?;

    let state = host.state();
    for freeze in &freezes {
        state
            .token(&freeze.token_id)
            .ok_or(Error::InvalidTokenId)?
            .main()
            .ok_or(Error::InvalidTokenId)?;
    }

    let state = host.state_mut();
    let mut owner = state
        .address_mut(&owner_address)
        .ok_or(Error::InvalidAddress)?;
    let owner = owner.holder_mut().ok_or(Error::InvalidAddress)?;
    let owner = owner.active_mut().ok_or(Error::RecoveredAddress)?;

    for FreezeParam {
        token_id,
        token_amount,
    } in freezes
    {
        ensure!(token_amount.gt(&TokenAmount::zero()), Error::InvalidAmount);
        owner
            .balance_mut(&token_id)
            .ok_or(Error::InsufficientFunds)?
            .un_freeze(token_amount)?;
        logger.log(&Event::TokenUnFrozen(TokenFrozen {
            token_id,
            amount: token_amount,
            address: owner_address,
        }))?
    }

    Ok(())
}

/// Returns the frozen balance of the given token for the given addresses.
///
/// # Returns
///
/// Returns `ContractResult<BalanceOfQueryResponse>` containing the frozen balance of the given token for the given addresses.
///
/// # Errors
///
/// - `Error::TokenDoesNotExist`: If any of the specified tokens do not exist.
/// - `Error::InvalidAddress`: If any of the provided addresses are invalid.
/// - `Error::ParseError`: If the input parameters could not be parsed correctly.
#[receive(
    contract = "security_sft_rewards",
    name = "balanceOfFrozen",
    parameter = "BalanceOfQueryParams",
    return_value = "BalanceOfQueryResponse",
    error = "Error"
)]
pub fn balance_of_frozen(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<BalanceOfQueryResponse> {
    let BalanceOfQueryParams { queries } = ctx.parameter_cursor().get()?;

    let mut amounts = Vec::with_capacity(queries.len());
    let state = host.state();
    for query in queries {
        state.token(&query.token_id).ok_or(Error::InvalidTokenId)?;
        let balance = {
            match state.address(&query.address) {
                None => TokenAmount::zero(),
                Some(address) => match address.holder() {
                    None => TokenAmount::zero(),
                    Some(holder) => match holder.active() {
                        None => TokenAmount::zero(),
                        Some(active) => active
                            .balance(&query.token_id)
                            .map(|b| b.frozen)
                            .unwrap_or(TokenAmount::zero()),
                    },
                },
            }
        };
        amounts.push(balance);
    }

    Ok(concordium_cis2::BalanceOfQueryResponse(amounts))
}

/// Returns the unfrozen balance of the given token for the given addresses.
///
/// # Returns
///
/// Returns `ContractResult<BalanceOfQueryResponse>` containing the unfrozen balance of the given token for the given addresses.
///
/// # Errors
///
/// Returns `Error::TokenDoesNotExist` if the token does not exist.
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_rewards",
    name = "balanceOfUnFrozen",
    parameter = "BalanceOfQueryParams",
    return_value = "BalanceOfQueryResponse",
    error = "Error"
)]
pub fn balance_of_un_frozen(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<BalanceOfQueryResponse> {
    let BalanceOfQueryParams { queries } = ctx.parameter_cursor().get()?;

    let mut amounts = Vec::with_capacity(queries.len());
    let state = host.state();
    for query in queries {
        state.token(&query.token_id).ok_or(Error::InvalidTokenId)?;
        let balance = {
            match state.address(&query.address) {
                None => TokenAmount::zero(),
                Some(address) => match address.holder() {
                    None => TokenAmount::zero(),
                    Some(holder) => match holder.active() {
                        None => TokenAmount::zero(),
                        Some(active) => active
                            .balance(&query.token_id)
                            .map(|b| b.un_frozen)
                            .unwrap_or(TokenAmount::zero()),
                    },
                },
            }
        };
        amounts.push(balance);
    }

    Ok(concordium_cis2::BalanceOfQueryResponse(amounts))
}
