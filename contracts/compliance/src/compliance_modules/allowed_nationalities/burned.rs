use concordium_std::*;

use super::state::State;
use super::types::*;

#[receive(
    contract = "rwa_compliance_module_allowed_nationalities",
    name = "burned",
    error = "super::types::Error"
)]
fn burned(_: &ReceiveContext, _: &Host<State>) -> ContractResult<()> { Ok(()) }
