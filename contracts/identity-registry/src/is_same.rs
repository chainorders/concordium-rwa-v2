use concordium_std::*;

use super::error::Error;
use super::state::State;
use super::types::ContractResult;

/// Handles the `isSame` contract call in the `rwa_identity_registry` contract.
///
/// This function is called to check if two addresses are associated with the
/// same identity. It retrieves the identities associated with the addresses
/// from the state, and checks if they are equal.
///
/// # Errors
///
/// Returns `Error::IdentityNotFound` if the identity associated with either of
/// the addresses could not be found. Returns `Error::ParseError` if the
/// parameters could not be parsed.
#[receive(
    contract = "rwa_identity_registry",
    name = "isSame",
    parameter = "(Address, Address)",
    return_value = "bool",
    error = "super::error::Error"
)]
fn is_same(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<bool> {
    let (address1, address2): (Address, Address) = ctx.parameter_cursor().get()?;
    let identity_1 = host
        .state
        .identities
        .get(&address1)
        .ok_or(Error::IdentityNotFound)?;
    let identity_2 = host
        .state
        .identities
        .get(&address2)
        .ok_or(Error::IdentityNotFound)?;

    Ok(identity_1.eq(&identity_2))
}
