use concordium_protocols::concordium_cis4::cis4_client;
use concordium_std::*;

use super::error::*;
use super::event::*;
use super::state::State;
use super::types::*;

/// Returns true if the given address is an issuer.
///
/// # Returns
///
/// Returns `ContractResult<Vec<ContractAddress>>` containing the list of
/// issuers.
#[receive(
    contract = "rwa_identity_registry",
    name = "isIssuer",
    parameter = "ContractAddress",
    error = "super::error::Error",
    return_value = "bool"
)]
pub fn is_issuer(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<bool> {
    let address: ContractAddress = ctx.parameter_cursor().get()?;
    Ok(host.state().issuers.contains(&address))
}

#[receive(
    contract = "rwa_identity_registry",
    name = "issuers",
    return_value = "Vec<Issuer>",
    error = "super::error::Error"
)]
/// Returns the list of issuers.
pub fn issuers(_ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<Vec<Issuer>> {
    Ok(host.state().issuers.iter().map(|a| *a).collect())
}

/// Adds the given address as an issuer.
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
    contract = "rwa_identity_registry",
    name = "addIssuer",
    mutable,
    enable_logger,
    parameter = "Issuer",
    error = "super::error::Error"
)]
pub fn add_issuer(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        Error::Unauthorized
    );
    let issuer: Issuer = ctx.parameter_cursor().get()?;
    ensure!(
        cis4_client::supports_cis4(host, issuer)?,
        Error::InvalidIssuer
    );
    ensure!(
        host.state_mut().issuers.insert(issuer),
        Error::IssuerAlreadyExists
    );
    logger.log(&Event::IssuerAdded(IssuerUpdatedEvent { issuer }))?;

    Ok(())
}

/// Removes the given address as an issuer.
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
    contract = "rwa_identity_registry",
    name = "removeIssuer",
    mutable,
    enable_logger,
    parameter = "Issuer",
    error = "super::error::Error"
)]
pub fn remove_issuer(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(
        ctx.sender().matches_account(&ctx.owner()),
        Error::Unauthorized
    );
    let issuer: ContractAddress = ctx.parameter_cursor().get()?;
    ensure!(
        host.state_mut().issuers.remove(&issuer),
        Error::IssuerNotFound
    );
    logger.log(&Event::IssuerRemoved(IssuerUpdatedEvent { issuer }))?;

    Ok(())
}
