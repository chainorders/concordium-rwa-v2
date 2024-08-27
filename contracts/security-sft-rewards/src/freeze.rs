use concordium_protocols::concordium_cis2_security::TokenFrozen;
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_rwa_utils::state_implementations::holders_security_state::IHoldersSecurityState;
use concordium_rwa_utils::state_implementations::tokens_state::{ITokensState, TokenStateResult};
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
    let state = host.state_mut();
    // Sender of this transaction should be a Trusted Agent
    ensure!(
        state.is_agent(&ctx.sender(), vec![AgentRole::Freeze]),
        Error::Unauthorized
    );
    let FreezeParams { owner, tokens }: FreezeParams = ctx.parameter_cursor().get()?;
    for token in tokens {
        ensure!(
            token.token_id.eq(&state.tracked_token_id),
            Error::InvalidTokenId
        );
        state.freeze(owner, &token.token_id, &token.token_amount)?;
        logger.log(&Event::TokenFrozen(TokenFrozen {
            token_id: token.token_id,
            amount:   token.token_amount,
            address:  owner,
        }))?;
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
    let state = host.state_mut();
    // Sender of this transaction should be a Trusted Agent
    ensure!(
        state.is_agent(&ctx.sender(), vec![AgentRole::UnFreeze]),
        Error::Unauthorized
    );
    let FreezeParams { owner, tokens }: FreezeParams = ctx.parameter_cursor().get()?;
    for token in tokens {
        ensure!(
            token.token_id.eq(&state.tracked_token_id),
            Error::InvalidTokenId
        );
        state.un_freeze(&owner, &token.token_id, &token.token_amount)?;
        logger.log(&Event::TokenUnFrozen(TokenFrozen {
            token_id: token.token_id,
            amount:   token.token_amount,
            address:  owner,
        }))?;
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
    let state = host.state();
    let BalanceOfQueryParams { queries } = ctx.parameter_cursor().get()?;

    let amounts = queries
        .iter()
        .map(|query| {
            state
                .ensure_token_exists(&query.token_id)
                .map(|_| state.balance_of_frozen(&query.address, &query.token_id))
        })
        .collect::<TokenStateResult<Vec<_>>>()?;

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
    let state = host.state();
    let BalanceOfQueryParams { queries } = ctx.parameter_cursor().get()?;

    let amounts = queries
        .iter()
        .map(|query| {
            state
                .ensure_token_exists(&query.token_id)
                .map(|_| state.balance_of_unfrozen(&query.address, &query.token_id))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(concordium_cis2::BalanceOfQueryResponse(amounts))
}
