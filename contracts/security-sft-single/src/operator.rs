use concordium_cis2::*;
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
    contract = "security_sft_single",
    name = "updateOperator",
    mutable,
    enable_logger,
    parameter = "UpdateOperatorParams",
    error = "Error"
)]
pub fn update_operator(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let UpdateOperatorParams { 0: updates }: UpdateOperatorParams = ctx.parameter_cursor().get()?;
    let sender = ctx.sender();
    let (state, state_builder) = host.state_and_builder();

    for UpdateOperator { operator, update } in updates {
        let mut holder = state.address_or_insert_holder(&sender, state_builder);
        let holder = holder.holder_mut().ok_or(Error::InvalidAddress)?;
        let holder = holder.active_mut().ok_or(Error::RecoveredAddress)?;

        match update {
            OperatorUpdate::Add => holder.add_operator(operator),
            OperatorUpdate::Remove => holder.remove_operator(&operator),
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
    contract = "security_sft_single",
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
    let mut res = Vec::with_capacity(queries.len());

    for query in queries {
        let is_operator = state.address(&query.owner).map_or(false, |a| {
            a.holder().map_or(false, |h| {
                h.active().map_or(false, |a| a.has_operator(&query.address))
            })
        });
        res.push(is_operator);
    }

    Ok(OperatorOfQueryResponse(res))
}
