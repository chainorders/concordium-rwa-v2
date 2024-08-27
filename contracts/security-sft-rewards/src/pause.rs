use concordium_protocols::concordium_cis2_security::Paused;
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_rwa_utils::state_implementations::tokens_security_state::ITokensSecurityState;
use concordium_rwa_utils::state_implementations::tokens_state::ITokensState;
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
    contract = "security_sft_rewards",
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
    ensure!(
        state.is_agent(&ctx.sender(), vec![AgentRole::Pause]),
        Error::Unauthorized
    );
    let PauseParams { tokens }: PauseParams = ctx.parameter_cursor().get()?;
    for token_id in tokens {
        ensure!(token_id.eq(&state.tracked_token_id), Error::InvalidTokenId);
        state.pause(token_id)?;
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
    contract = "security_sft_rewards",
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
    ensure!(
        state.is_agent(&ctx.sender(), vec![AgentRole::UnPause]),
        Error::Unauthorized
    );
    let PauseParams { tokens }: PauseParams = ctx.parameter_cursor().get()?;
    for token_id in tokens {
        ensure!(token_id.eq(&state.tracked_token_id), Error::InvalidTokenId);
        state.un_pause(token_id)?;
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
    contract = "security_sft_rewards",
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
    for token_id in tokens {
        state.ensure_token_exists(&token_id)?;
        res.tokens.push(state.is_paused(&token_id))
    }

    Ok(res)
}
