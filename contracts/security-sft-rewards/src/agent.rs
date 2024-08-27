use concordium_protocols::concordium_cis2_security::AgentUpdatedEvent;
use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::{Agent, ContractResult, Event};
use crate::state::{AddressState, AgentState};
/// Returns true if the given address is an agent.
///
/// # Returns
///
/// Returns `ContractResult<Vec<Address>>` containing the list of agents.
#[receive(
    contract = "security_sft_rewards",
    name = "isAgent",
    parameter = "Agent",
    return_value = "bool",
    error = "Error"
)]
pub fn is_agent(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<bool> {
    let agent: Agent = ctx.parameter_cursor().get()?;
    let is_agent = host
        .state()
        .address(&agent.address)
        .is_some_and(|a| a.is_agent(&agent.roles));
    Ok(is_agent)
}

/// Adds the given address as an agent.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was
/// successful.
///
/// # Errors
///
/// Returns `Error::Unauthorized` if the sender does not match the owner.
#[receive(
    contract = "security_sft_rewards",
    name = "addAgent",
    mutable,
    enable_logger,
    parameter = "Agent",
    error = "Error"
)]
pub fn add_agent(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: Agent = ctx.parameter_cursor().get()?;
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        Error::Unauthorized
    );
    host.state_mut().add_address(
        params.address,
        AddressState::Agent(AgentState(params.roles.clone())),
    )?;
    logger.log(&Event::AgentAdded(AgentUpdatedEvent {
        agent: params.address,
        roles: params.roles,
    }))?;

    Ok(())
}

/// Removes the given address as an agent.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was successful.
///
/// # Errors
///
/// Returns `Error::Unauthorized` if the sender does not match the owner.
#[receive(
    contract = "security_sft_rewards",
    name = "removeAgent",
    mutable,
    enable_logger,
    parameter = "Address",
    error = "Error"
)]
pub fn remove_agent(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        Error::Unauthorized
    );
    let address: Address = ctx.parameter_cursor().get()?;
    let state = host.state_mut().remove_and_get_address(&address)?;
    let agent = state.agent().ok_or(Error::InvalidAddress)?;
    logger.log(&Event::AgentRemoved(AgentUpdatedEvent {
        agent: address,
        roles: agent.roles().clone(),
    }))?;

    Ok(())
}
