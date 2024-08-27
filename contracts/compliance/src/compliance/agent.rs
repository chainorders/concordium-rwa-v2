use concordium_rwa_utils::state_implementations::agents_state::IsAgentsState;
use concordium_std::*;

use super::error::*;
use super::event::*;
use super::state::State;
use super::types::*;

/// Returns true if the given address is an agent.
///
/// # Returns
///
/// Returns `ContractResult<Vec<Address>>` containing the list of agents.
#[receive(
    contract = "rwa_compliance",
    name = "isAgent",
    return_value = "bool",
    parameter = "Address",
    error = "super::error::Error"
)]
pub fn is_agent(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<bool> {
    let address: Address = ctx.parameter_cursor().get()?;
    Ok(host.state().is_agent(&address))
}

#[receive(
    contract = "rwa_compliance",
    name = "agents",
    return_value = "Vec<Address>",
    error = "super::error::Error"
)]
/// Returns the list of agents.
pub fn agents(_ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<Vec<Address>> {
    Ok(host.state().list_agents())
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
    contract = "rwa_compliance",
    name = "addAgent",
    mutable,
    enable_logger,
    parameter = "Address",
    error = "super::error::Error"
)]
pub fn add_agent(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        Error::Unauthorized
    );
    let agent: Address = ctx.parameter_cursor().get()?;
    ensure!(host.state_mut().add_agent(agent), Error::AgentAlreadyExists);
    logger.log(&Event::AgentAdded(AgentUpdatedEvent { agent }))?;

    Ok(())
}

/// Removes the given address as an agent.
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
    contract = "rwa_compliance",
    name = "removeAgent",
    mutable,
    enable_logger,
    parameter = "Address",
    error = "super::error::Error"
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
    let agent: Address = ctx.parameter_cursor().get()?;
    ensure!(host.state_mut().remove_agent(&agent), Error::AgentNotFound);
    logger.log(&Event::AgentRemoved(AgentUpdatedEvent { agent }))?;

    Ok(())
}
