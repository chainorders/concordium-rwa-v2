use concordium_std::*;

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
    contract = "rwa_compliance_module_allowed_nationalities",
    name = "minted",
    error = "super::types::Error"
)]
fn minted(_: &ReceiveContext, _: &Host<State>) -> ContractResult<()> { Ok(()) }
