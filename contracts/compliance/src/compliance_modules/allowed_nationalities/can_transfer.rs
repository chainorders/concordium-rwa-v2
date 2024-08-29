use concordium_protocols::concordium_cis2_security::{
    identity_registry_client, CanTransferParam, IdentityAttribute,
};
use concordium_std::attributes::NATIONALITY;
use concordium_std::*;

use super::state::State;
use super::types::*;

#[receive(
    contract = "rwa_compliance_module_allowed_nationalities",
    name = "canTransfer",
    parameter = "CanTransferParam<TokenId, TokenAmount>",
    return_value = "bool",
    error = "super::types::Error"
)]
fn can_transfer(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<bool> {
    let params: CanTransferParam<TokenId, TokenAmount> = ctx.parameter_cursor().get()?;
    let state = host.state();

    let id = identity_registry_client::get_identity(host, &state.identity_registry(), params.to)?;
    for IdentityAttribute { tag, value } in id.attributes {
        if tag.eq(&NATIONALITY.0) {
            return Ok(state.is_allowed(value));
        }
    }

    Ok(false)
}
