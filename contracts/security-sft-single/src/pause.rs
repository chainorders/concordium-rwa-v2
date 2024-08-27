use concordium_protocols::concordium_cis2_security::Paused;
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::*;

/// Pauses the given tokenIds.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was successful.
///
/// # Errors
///
/// Returns `Error::Unauthorized` if the sender is not an agent.
/// Returns `Error::TokenDoesNotExist` if the token does not exist.
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_single",
    name = "pause",
    mutable,
    enable_logger,
    parameter = "PauseParams",
    error = "super::error::Error"
)]
pub fn pause(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let state = host.state_mut();
    let is_authorized = state
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::Pause]));
    ensure!(is_authorized, Error::Unauthorized);

    let PauseParams { tokens }: PauseParams = ctx.parameter_cursor().get()?;
    for token_id in tokens {
        state.token.pause();
        logger.log(&Event::Paused(Paused { token_id }))?;
    }

    Ok(())
}

/// Unpauses the given tokens.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was successful.
///
/// # Errors
///
/// Returns `Error::Unauthorized` if the sender is not an agent.
/// Returns `Error::TokenDoesNotExist` if the token does not exist.
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_single",
    name = "unPause",
    mutable,
    enable_logger,
    parameter = "PauseParams",
    error = "super::error::Error"
)]
pub fn un_pause(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let state = host.state_mut();
    let is_authorized = state
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::UnPause]));
    ensure!(is_authorized, Error::Unauthorized);

    let PauseParams { tokens }: PauseParams = ctx.parameter_cursor().get()?;
    for token_id in tokens {
        state.token.un_pause();
        logger.log(&Event::UnPaused(Paused { token_id }))?;
    }

    Ok(())
}

/// Returns true if the given tokens are paused.
///
/// # Returns
///
/// Returns `ContractResult<IsPausedResponse>` containing a boolean for each token indicating whether it is paused.
///
/// # Errors
///
/// Returns `Error::TokenDoesNotExist` if the token does not exist.
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_single",
    name = "isPaused",
    parameter = "PauseParams",
    return_value = "IsPausedResponse",
    error = "super::error::Error"
)]
pub fn is_paused(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<IsPausedResponse> {
    let PauseParams { tokens }: PauseParams = ctx.parameter_cursor().get()?;
    let mut res = IsPausedResponse {
        tokens: Vec::with_capacity(tokens.len()),
    };

    let state = host.state();
    for _ in tokens {
        res.tokens.push(state.token.paused);
    }

    Ok(res)
}
