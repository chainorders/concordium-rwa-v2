use concordium_protocols::concordium_cis2_security::AgentUpdatedEvent;
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::{Agent, AgentRole, ContractResult, Event};
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
    Ok(host.state().is_agent(&agent.address, agent.roles))
}

#[receive(
    contract = "security_sft_rewards",
    name = "agents",
    return_value = "Vec<Agent>",
    error = "Error"
)]
/// Returns the list of agents.
pub fn agents(_ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<Vec<Agent>> {
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
    let (state, state_builder) = host.state_and_builder();
    ensure!(
        state.is_agent(
            &ctx.sender(),
            [params.roles.as_slice(), &[AgentRole::AddAgent]].concat(),
        ),
        Error::Unauthorized
    );
    ensure!(
        state.add_agent(params.to_owned(), state_builder),
        Error::AgentAlreadyExists
    );
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
    let agent: Option<Agent> = host.state_mut().remove_agent(&address);
    match agent {
        None => bail!(Error::AgentNotFound),
        Some(agent) => {
            logger.log(&Event::AgentRemoved(AgentUpdatedEvent {
                agent: agent.address,
                roles: agent.roles,
            }))?;
        }
    };

    Ok(())
}
