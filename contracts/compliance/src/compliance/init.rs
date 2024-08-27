use concordium_rwa_utils::state_implementations::agents_state::IsAgentsState;
use concordium_std::*;

use super::event::Event;
use super::state::State;
use super::types::{ContractResult, Module};

#[derive(Serialize, SchemaType)]
pub struct InitParams {
    pub modules: Vec<Module>,
}

#[init(
    contract = "rwa_compliance",
    event = "super::event::Event",
    error = "super::error::Error",
    parameter = "InitParams",
    enable_logger
)]
pub fn init(
    ctx: &InitContext,
    state_builder: &mut StateBuilder,
    logger: &mut Logger,
) -> ContractResult<State> {
    let params: InitParams = ctx.parameter_cursor().get()?;
    for module in params.modules.iter() {
        logger.log(&Event::ModuleAdded(module.to_owned()))?;
    }
    let mut state = State::new(params.modules, state_builder);
    state.add_agent(Address::Account(ctx.init_origin()));

    Ok(state)
}
