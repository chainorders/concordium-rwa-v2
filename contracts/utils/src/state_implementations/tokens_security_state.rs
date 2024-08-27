use concordium_cis2::IsTokenId;
use concordium_std::*;

use super::tokens_state::{ITokenState, ITokensState};

pub enum TokenSecurityError {
    /// The token is paused.
    PausedToken,
    InvalidTokenId,
}

pub trait ISecurityTokenState: ITokenState {
    /// Returns `true` if the token is paused, `false` otherwise.
    fn paused(&self) -> bool;
    /// Sets the `is_paused` field to the given value.
    fn set_paused(&mut self, is_paused: bool);
}

/// The `ITokensSecurityState` trait defines the interface for managing the
/// security state of tokens.
pub trait ITokensSecurityState<T: IsTokenId, TTokenState: ISecurityTokenState, S: HasStateApi>:
    ITokensState<T, TTokenState, S> {
    /// Checks if the token with the given ID is paused.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The ID of the token to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the token is paused, `false` otherwise. If the token
    /// does not exist, returns `false`.
    fn is_paused(&self, token_id: &T) -> bool {
        self.tokens()
            .get(token_id)
            .map(|t| t.paused())
            .unwrap_or(false)
    }

    /// Ensures that the given token is not paused.
    ///
    /// # Errors
    ///
    /// Returns a `TokenSecurityError::TokenPaused` error if the token is
    /// paused.
    fn ensure_not_paused(&self, token_id: &T) -> Result<(), TokenSecurityError> {
        if self.is_paused(token_id) {
            Err(TokenSecurityError::PausedToken)
        } else {
            Ok(())
        }
    }

    /// Pauses the token with the given ID.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The ID of the token to pause.
    ///
    /// If the token does not exist, it is created and then paused.
    fn pause(&mut self, token_id: T) -> Result<(), TokenSecurityError> {
        self.tokens_mut()
            .entry(token_id)
            .occupied_or(TokenSecurityError::InvalidTokenId)?
            .modify(|t| t.set_paused(true));

        Ok(())
    }

    /// Unpauses the token with the given ID.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The ID of the token to unpause.
    ///
    /// If the token does not exist, nothing happens.
    fn un_pause(&mut self, token_id: T) -> Result<(), TokenSecurityError> {
        self.tokens_mut()
            .entry(token_id)
            .occupied_or(TokenSecurityError::InvalidTokenId)?
            .modify(|t: &mut TTokenState| t.set_paused(false));

        Ok(())
    }
}
