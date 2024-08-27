use concordium_std::*;

use super::error::Error;
use super::event::Event;
use super::state::State;
use super::types::ContractResult;

#[receive(
    contract = "rwa_compliance",
    name = "addModule",
    parameter = "ContractAddress",
    error = "super::error::Error",
    mutable,
    enable_logger
)]
pub fn add_module(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(
        host.state().agents.contains(&ctx.sender()),
        Error::Unauthorized
    );
    let module: ContractAddress = ctx.parameter_cursor().get()?;
    host.state_mut().add_module(module);
    logger.log(&Event::ModuleAdded(module))?;
    Ok(())
}

#[receive(
    contract = "rwa_compliance",
    name = "removeModule",
    parameter = "ContractAddress",
    error = "super::error::Error",
    mutable,
    enable_logger
)]
pub fn remove_module(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure!(
        host.state().agents.contains(&ctx.sender()),
        Error::Unauthorized
    );
    let module: ContractAddress = ctx.parameter_cursor().get()?;
    host.state_mut().remove_module(&module);
    logger.log(&Event::ModuleRemoved(module))?;
    Ok(())
}

#[receive(
    contract = "rwa_compliance",
    name = "modules",
    return_value = "Vec<ContractAddress>"
)]
pub fn modules(_: &ReceiveContext, host: &Host<State>) -> ContractResult<Vec<ContractAddress>> {
    Ok(host.state().modules())
}
