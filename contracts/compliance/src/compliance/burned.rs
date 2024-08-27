use concordium_protocols::concordium_cis2_security::{compliance_client, BurnedParam};
use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::*;

/// Handles the `burned` event in the `rwa_compliance` contract.
///
/// This function is called when tokens are burned. It iterates over all modules
/// in the state, and calls the `burned` function on the `ComplianceContract`
/// for each module.
///
/// # Errors
///
/// Returns `Error::ParseError` if the parameters could not be parsed.
/// Returns `Error::Unauthorized` if the sender of the event does not match the
/// contract of the token.
#[receive(
    contract = "rwa_compliance",
    name = "burned",
    parameter = "BurnedParam<TokenId, TokenAmount>",
    error = "super::error::Error",
    mutable
)]
fn burned(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let params: BurnedParam<TokenId, TokenAmount> = ctx.parameter_cursor().get()?;
    ensure!(
        ctx.sender().matches_contract(&params.token_id.contract),
        Error::Unauthorized
    );

    let modules = host.state().modules();
    for module in modules {
        compliance_client::burned(host, &module, &params)?;
    }

    Ok(())
}
