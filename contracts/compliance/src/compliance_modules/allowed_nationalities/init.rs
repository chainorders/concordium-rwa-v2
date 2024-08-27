use concordium_std::*;

use super::state::State;
use super::types::{AttributeValue, ContractResult};

#[derive(Serialize, SchemaType)]
pub struct InitParams {
    pub nationalities:     Vec<AttributeValue>,
    pub identity_registry: ContractAddress,
}

#[init(
    contract = "rwa_compliance_module_allowed_nationalities",
    error = "super::types::Error",
    parameter = "InitParams"
)]
pub fn init(ctx: &InitContext, state_builder: &mut StateBuilder) -> ContractResult<State> {
    let params: InitParams = ctx.parameter_cursor().get()?;

    Ok(State::new(
        params.identity_registry,
        params.nationalities,
        state_builder,
    ))
}
