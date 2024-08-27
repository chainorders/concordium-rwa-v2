use concordium_protocols::concordium_cis4::{self, cis4_client};
use concordium_std::*;

use super::state::State;
use super::types::ContractResult;

/// Handles the `isVerified` contract call in the `rwa_identity_registry`
/// contract.
///
/// This function is called to check if an address is associated with a verified
/// identity. It retrieves the identity associated with the address from the
/// state, and checks the status of all credentials associated with the
/// identity. If all credentials are active, the identity is considered
/// verified.
///
/// # Errors
///
/// Returns `Error::IdentityNotFound` if the identity associated with the
/// address could not be found. Returns `Error::ParseError` if the parameters
/// could not be parsed.
#[receive(
    contract = "rwa_identity_registry",
    name = "isVerified",
    parameter = "Address",
    return_value = "bool",
    error = "super::error::Error"
)]
pub fn is_verified(ctx: &ReceiveContext, host: &Host<State>) -> ContractResult<bool> {
    let address: Address = ctx.parameter_cursor().get()?;

    // Check that the identity exists.
    let identity = host.state().identities.get(&address);
    let identity = match identity {
        Some(identity) => identity,
        None => return Ok(false),
    };

    let issuers = host.state().issuers.iter().map(|i| *i);
    for issuer in issuers {
        let credential_id = identity.credential_id(&issuer);
        let credential_status = match credential_id {
            Some(credential_holder_id) => {
                cis4_client::credential_status(host, issuer, credential_holder_id)?
            }
            None => return Ok(false),
        };

        // If the credential is not active, the identity is not verified.
        if credential_status.ne(&concordium_cis4::CredentialStatus::Active) {
            return Ok(false);
        }
    }

    Ok(true)
}
