use concordium_protocols::concordium_cis2_security::{identity_registry_client, RecoverEvent};
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::{AgentRole, ContractResult, Event, RecoverParam};
use crate::state::{AddressState, HolderAddressState};

#[receive(
    contract = "security_sft_single",
    name = "recover",
    mutable,
    enable_logger,
    parameter = "RecoverParam",
    error = "Error"
)]
pub fn recover(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let RecoverParam {
        lost_account,
        new_account,
    }: RecoverParam = ctx.parameter_cursor().get()?;
    let state = host.state();
    let is_authorized = state
        .address(&ctx.sender())
        .is_some_and(|a| a.is_agent(&[AgentRole::HolderRecovery]));
    ensure!(is_authorized, Error::Unauthorized);
    ensure!(
        identity_registry_client::is_verified(host, &state.identity_registry, &new_account)?,
        Error::UnVerifiedIdentity
    );

    let (state, state_builder) = host.state_and_builder();
    let lost_holder = state
        .address(&lost_account)
        .ok_or(Error::InvalidAddress)?
        .holder()
        .ok_or(Error::InvalidAddress)?
        .active()
        .ok_or(Error::RecoveredAddress)?
        .clone_for_recovery(state_builder);
    state.add_address(
        new_account,
        AddressState::Holder(HolderAddressState::Holder(lost_holder)),
    )?;
    let mut lost_holder = state
        .address_mut(&lost_account)
        .ok_or(Error::InvalidAddress)?;
    let lost_holder = lost_holder.holder_mut().ok_or(Error::InvalidAddress)?;
    *lost_holder = HolderAddressState::Recovered(new_account);
    logger.log(&Event::Recovered(RecoverEvent {
        lost_account,
        new_account,
    }))?;

    Ok(())
}

#[receive(
    contract = "security_sft_single",
    name = "recoveryAddress",
    parameter = "Address",
    error = "Error",
    return_value = "Option<Address>"
)]
pub fn recovery_address(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<Option<Address>> {
    let address: Address = ctx.parameter_cursor().get()?;
    let recovery_address = host
        .state()
        .address(&address)
        .and_then(|a| a.holder().and_then(|h| h.recovered().cloned()));
    Ok(recovery_address)
}
