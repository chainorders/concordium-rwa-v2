use concordium_cis2::*;
use concordium_std::*;

pub enum TokenStateError {
    InvalidTokenId,
}

pub type TokenStateResult<T> = Result<T, TokenStateError>;

pub trait ITokenState: Serialize+Clone {
    fn metadata_url(&self) -> &MetadataUrl;
}

pub trait ITokensState<T: IsTokenId, TTokenState: ITokenState, S: HasStateApi> {
    fn tokens(&self) -> &StateMap<T, TTokenState, S>;
    fn tokens_mut(&mut self) -> &mut StateMap<T, TTokenState, S>;

    fn token_exists(&self, token_id: &T) -> bool { self.tokens().get(token_id).is_some() }

    /// Checks if the token with the given ID exists.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The ID of the token to check.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the token exists,
    /// `Err(TokenStateError::TokenDoesNotExist)` otherwise.
    fn ensure_token_exists(&self, token_id: &T) -> TokenStateResult<()> {
        ensure!(self.token_exists(token_id), TokenStateError::InvalidTokenId);
        Ok(())
    }

    fn token(&self, token_id: &T) -> TokenStateResult<TTokenState> {
        self.tokens()
            .get(token_id)
            .map(|token| token.clone())
            .ok_or(TokenStateError::InvalidTokenId)
    }

    /// Adds a token with the given ID and metadata URL to the state.
    ///
    /// # Arguments
    ///
    /// * `token_id` - The ID of the token to add.
    /// * `metadata_url` - The metadata URL of the token to add.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the token was added successfully,
    /// `Err(TokenStateError::TokenAlreadyExists)` if the token already exists.
    fn add_token(&mut self, token_id: T, state: TTokenState) -> TokenStateResult<()> {
        self.tokens_mut()
            .entry(token_id)
            .vacant_or(TokenStateError::InvalidTokenId)?
            .insert(state);

        Ok(())
    }

    fn metadata_url(&self, token_id: &T) -> TokenStateResult<MetadataUrl> {
        self.tokens()
            .get(token_id)
            .map(|t| t.metadata_url().clone())
            .ok_or(TokenStateError::InvalidTokenId)
    }
}
