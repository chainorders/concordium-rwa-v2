use concordium_protocols::concordium_cis2_security::{compliance_client, MintedParam};
use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::*;

/// Handles the `minted` event in the `rwa_compliance` contract.
///
/// This function is called when tokens are minted. It iterates over all modules
/// in the state, and calls the `minted` function on the `ComplianceContract`
/// for each module.
///
/// # Errors
/// Returns `Error::ParseError` if the parameters could not be parsed.
/// Returns `Error::Unauthorized` if the sender of the event does not match the
/// contract of the token.
#[receive(
    contract = "rwa_compliance",
    name = "minted",
    parameter = "MintedParam<TokenId, TokenAmount>",
    error = "super::error::Error",
    mutable
)]
fn minted(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let params: MintedParam<TokenId, TokenAmount> = ctx.parameter_cursor().get()?;
    for module in host.state.modules().iter() {
        ensure!(
            ctx.sender().matches_contract(&params.token_id.contract),
            Error::Unauthorized
        );
        compliance_client::minted(host, module, &params)?;
    }

    Ok(())
}
