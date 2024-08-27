use concordium_std::*;

use super::state::State;
use super::types::*;

#[receive(
    contract = "rwa_compliance_module_allowed_nationalities",
    name = "transferred",
    error = "super::types::Error"
)]
fn transferred(_: &ReceiveContext, _: &Host<State>) -> ContractResult<()> { Ok(()) }
