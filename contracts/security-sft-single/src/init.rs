use concordium_cis2::{Cis2Event, TokenIdUnit, TokenMetadataEvent};
use concordium_protocols::concordium_cis2_security::{
    AgentUpdatedEvent, ComplianceAdded, IdentityRegistryAdded,
};
use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::{AgentRole, ContractResult, Event, InitParam};
use crate::state::{AddressState, AgentState, TokenState};
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
    contract = "security_sft_single",
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
    let owner = Address::Account(ctx.init_origin());
    let addresses = {
        let mut addresses = state_builder.new_map();
        let _ = addresses.insert(
            owner,
            AddressState::Agent(AgentState(AgentRole::owner_roles())),
        );
        addresses
    };
    let state = State {
        identity_registry: params.identity_registry,
        compliance: params.compliance,
        sponsor: params.sponsors,
        addresses,
        token: TokenState {
            metadata_url: params.metadata_url.into(),
            supply:       0.into(),
            paused:       false,
        },
    };

    logger.log(&Event::IdentityRegistryAdded(IdentityRegistryAdded(
        state.identity_registry,
    )))?;
    logger.log(&Event::ComplianceAdded(ComplianceAdded(state.compliance)))?;
    for (address, address_state) in state.addresses.iter() {
        if let Some(agent) = address_state.agent() {
            logger.log(&Event::AgentAdded(AgentUpdatedEvent {
                agent: *address,
                roles: agent.0.clone(),
            }))?;
        }
    }
    logger.log(&Event::Cis2(Cis2Event::TokenMetadata(TokenMetadataEvent {
        metadata_url: state.token.metadata_url().clone(),
        token_id:     TokenIdUnit(),
    })))?;

    Ok(state)
}

/// Returns the address of the identity registry contract.
///
/// # Returns
///
/// Returns `ContractResult<ContractAddress>` containing the address of the identity registry contract.
#[receive(
    contract = "security_sft_single",
    name = "identityRegistry",
    return_value = "ContractAddress"
)]
pub fn identity_registry(
    _: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<ContractAddress> {
    Ok(host.state().identity_registry)
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
    contract = "security_sft_single",
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
    let is_authorized = host
        .state()
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::SetIdentityRegistry]));
    ensure!(is_authorized, Error::Unauthorized);

    let identity_registry: ContractAddress = ctx.parameter_cursor().get()?;
    host.state_mut().identity_registry = identity_registry;
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
    contract = "security_sft_single",
    name = "compliance",
    return_value = "ContractAddress"
)]
pub fn compliance(_: &ReceiveContext, host: &Host<State>) -> ContractResult<ContractAddress> {
    Ok(host.state().compliance)
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
    contract = "security_sft_single",
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
    let is_authorized = host
        .state()
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::SetCompliance]));
    ensure!(is_authorized, Error::Unauthorized);

    let compliance: ContractAddress = ctx.parameter_cursor().get()?;
    host.state_mut().compliance = compliance;
    logger.log(&Event::ComplianceAdded(ComplianceAdded(compliance)))?;

    Ok(())
}
