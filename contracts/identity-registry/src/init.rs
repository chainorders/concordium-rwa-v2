use concordium_std::*;

use super::state::State;
use super::types::ContractResult;

/// Initializes the contract.
#[init(
    contract = "rwa_identity_registry",
    event = "super::event::Event",
    error = "super::error::Error"
)]
pub fn init(ctx: &InitContext, state_builder: &mut StateBuilder) -> ContractResult<State> {
    let owner = Address::Account(ctx.init_origin());

    Ok(State::new(vec![owner], state_builder))
}
