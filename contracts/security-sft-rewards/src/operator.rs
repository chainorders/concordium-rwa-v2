use concordium_cis2::*;
use concordium_rwa_utils::state_implementations::holders_state::IHoldersState;
use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::{ContractResult, Event};

/// Updates the operator status for the sender.
///
/// # Returns
///
/// Returns `ContractResult<()>` indicating whether the operation was successful.
///
/// # Errors
///
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_rewards",
    name = "updateOperator",
    mutable,
    enable_logger,
    parameter = "UpdateOperatorParams",
    error = "super::error::Error"
)]
pub fn update_operator(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let UpdateOperatorParams { 0: updates }: UpdateOperatorParams = ctx.parameter_cursor().get()?;
    let (state, state_builder) = host.state_and_builder();
    let sender = ctx.sender();
    ensure!(sender.is_account(), Error::InvalidAddress);
    for UpdateOperator { operator, update } in updates {
        match update {
            OperatorUpdate::Add => state.add_operator(sender, operator, state_builder),
            OperatorUpdate::Remove => state.remove_operator(sender, &operator),
        }
        logger.log(&Event::Cis2(Cis2Event::UpdateOperator(
            UpdateOperatorEvent {
                operator,
                update,
                owner: sender,
            },
        )))?;
    }
    Ok(())
}

/// # Returns
///
/// Returns `ContractResult<OperatorOfQueryResponse>` containing a boolean
/// indicating whether the given address is an operator for the given owner.
///
/// # Errors
///
/// Returns `Error::ParseError` if the parameters could not be parsed.
#[receive(
    contract = "security_sft_rewards",
    name = "operatorOf",
    parameter = "OperatorOfQueryParams",
    return_value = "OperatorOfQueryResponse",
    error = "super::error::Error"
)]
pub fn operator_of(
    ctx: &ReceiveContext,
    host: &Host<State>,
) -> ContractResult<OperatorOfQueryResponse> {
    let OperatorOfQueryParams { queries }: OperatorOfQueryParams = ctx.parameter_cursor().get()?;
    let state = host.state();
    let res: Vec<bool> = queries
        .iter()
        .map(|q| state.is_operator(&q.owner, &q.address))
        .collect();
    Ok(OperatorOfQueryResponse(res))
}
