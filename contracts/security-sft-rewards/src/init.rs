use concordium_cis2::{Cis2Event, TokenMetadataEvent};
use concordium_protocols::concordium_cis2_security::{
    AgentUpdatedEvent, ComplianceAdded, IdentityRegistryAdded,
};
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_rwa_utils::state_implementations::cis2_security_state::ICis2SecurityState;
use concordium_rwa_utils::state_implementations::tokens_state::ITokensState;
use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::{Agent, AgentRole, ContractResult, Event, InitParam};
/// Initializes the contract with the given parameters.
///
/// # Returns
///
/// Returns `InitResult<State>` indicating whether the operation was successful.
///
/// # Errors
///
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[init(
    contract = "security_sft_rewards",
    event = "Event",
    error = "Error",
    parameter = "InitParam",
    enable_logger
)]
pub fn init(
    ctx: &InitContext,
    state_builder: &mut StateBuilder,
    logger: &mut Logger,
) -> InitResult<State> {
    let params: InitParam = ctx.parameter_cursor().get()?;
    ensure!(
        params.min_reward_token_id.gt(&params.tracked_token_id),
        Error::InvalidTokenId.into()
    );
    let owner = Address::Account(ctx.init_origin());
    let state = State::new(
        params.identity_registry,
        params.compliance,
        params.sponsors,
        // Adds owner as an agent
        vec![Agent {
            address: owner,
            roles:   AgentRole::owner_roles(),
        }],
        params.metadata_url.into(),
        params.blank_reward_metadata_url.into(),
        params.tracked_token_id,
        params.min_reward_token_id,
        state_builder,
    );

    logger.log(&Event::IdentityRegistryAdded(IdentityRegistryAdded(
        state.identity_registry,
    )))?;
    logger.log(&Event::ComplianceAdded(ComplianceAdded(state.compliance)))?;
    for agent in state.list_agents().iter() {
        logger.log(&Event::AgentAdded(AgentUpdatedEvent {
            agent: agent.address,
            roles: agent.roles.clone(),
        }))?;
    }
    for token in state.tokens().iter() {
        logger.log(&Event::Cis2(Cis2Event::TokenMetadata(TokenMetadataEvent {
            metadata_url: token.1.metadata_url().clone(),
            token_id:     *token.0,
        })))?;
    }

    Ok(state)
}

/// Returns the address of the identity registry contract.
///
/// # Returns
///
/// Returns `ContractResult<ContractAddress>` containing the address of the identity registry contract.
#[receive(
    contract = "security_sft_rewards",
    name = "identityRegistry",
    return_value = "ContractAddress"
)]
pub fn identity_registry(
    _: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<ContractAddress> {
    Ok(host.state().identity_registry())
}

/// Sets the address of the identity registry contract.
///
/// # Parameters
///
/// - `ContractAddress`: The address of the identity registry contract.
///
/// # Errors
///
/// Returns an `Error::Unauthorized` error if the caller is not authorized to set the identity registry.
#[receive(
    contract = "security_sft_rewards",
    name = "setIdentityRegistry",
    mutable,
    enable_logger,
    parameter = "ContractAddress",
    error = "Error"
)]
pub fn set_identity_registry(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let identity_registry: ContractAddress = ctx.parameter_cursor().get()?;
    ensure!(
        host.state()
            .is_agent(&ctx.sender(), vec![AgentRole::SetIdentityRegistry]),
        Error::Unauthorized
    );

    host.state_mut().set_identity_registry(identity_registry);
    logger.log(&Event::IdentityRegistryAdded(IdentityRegistryAdded(
        identity_registry,
    )))?;
    Ok(())
}

/// Returns the address of the compliance contract.
///
/// # Returns
///
/// Returns `ContractResult<ContractAddress>` containing the address of the compliance contract.
#[receive(
    contract = "security_sft_rewards",
    name = "compliance",
    return_value = "ContractAddress"
)]
pub fn compliance(_: &ReceiveContext, host: &Host<State>) -> ContractResult<ContractAddress> {
    Ok(host.state().compliance())
}

/// Sets the compliance contract address.
///
/// This function allows authorized agents to set the compliance contract address for the security SFT rewards contract.
///
/// # Parameters
///
/// - `ContractAddress`: The address of the compliance contract.
///
/// # Errors
///
/// Returns an `Error::Unauthorized` error if the caller is not authorized to set the compliance contract.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was successful.
#[receive(
    contract = "security_sft_rewards",
    name = "setCompliance",
    mutable,
    enable_logger,
    parameter = "ContractAddress",
    error = "Error"
)]
pub fn set_compliance(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let compliance: ContractAddress = ctx.parameter_cursor().get()?;
    ensure!(
        host.state()
            .is_agent(&ctx.sender(), vec![AgentRole::SetCompliance]),
        Error::Unauthorized
    );

    host.state_mut().set_compliance(compliance);
    logger.log(&Event::ComplianceAdded(ComplianceAdded(compliance)))?;

    Ok(())
}
