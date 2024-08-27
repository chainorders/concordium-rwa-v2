use concordium_protocols::concordium_cis2_ext::{IsTokenAmount, IsTokenId};
use concordium_std::*;

use super::holders_state::{HolderStateError, IHolderState, IHoldersState};
use super::tokens_state::{ITokenState, ITokensState, TokenStateError};

pub type Cis2Result<R> = Result<R, Cis2StateError>;

pub enum Cis2StateError {
    InvalidTokenId,
    InsufficientFunds,
    InvalidAmount,
}

pub trait ICis2TokenState<A>: ITokenState {
    fn inc_supply(&mut self, amount: &A);
    fn dec_supply(&mut self, amount: &A);
    fn supply(&self) -> A;
}

pub trait ICis2State<
    T: IsTokenId,
    A: IsTokenAmount,
    TTokenState: ICis2TokenState<A>,
    THolderState: IHolderState<T, A, S>,
    S: HasStateApi,
>: IHoldersState<T, A, THolderState, S>+ITokensState<T, TTokenState, S> {
    fn mint(
        &mut self,
        token_id: &T,
        amount: &A,
        to: &Address,
        state_builder: &mut StateBuilder<S>,
    ) -> Result<(), Cis2StateError> {
        self.tokens_mut()
            .entry(token_id.clone())
            .occupied_or(Cis2StateError::InvalidTokenId)?
            .modify(|t| t.inc_supply(amount));

        self.add_balance(to, token_id, amount, state_builder);

        Ok(())
    }

    fn burn(&mut self, token_id: &T, amount: &A, from: &Address) -> Result<(), Cis2StateError> {
        self.tokens_mut()
            .entry(token_id.clone())
            .occupied_or(Cis2StateError::InvalidTokenId)?
            .modify(|t| t.dec_supply(amount));

        self.sub_balance(from, token_id, amount)?;
        Ok(())
    }

    fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        token_id: &T,
        amount: &A,
        state_builder: &mut StateBuilder<S>,
    ) -> Result<(), Cis2StateError> {
        self.ensure_token_exists(token_id)?;
        self.sub_balance(from, token_id, amount)?;
        self.add_balance(to, token_id, amount, state_builder);

        Ok(())
    }

    fn metadata_url(&self, token_id: &T) -> Result<MetadataUrl, Cis2StateError> {
        let metadata_url = ITokensState::metadata_url(self, token_id)?;
        Ok(metadata_url)
    }

    fn supply_of(&self, token_id: &T) -> Result<A, Cis2StateError> {
        let supply = self
            .tokens()
            .get(token_id)
            .ok_or(Cis2StateError::InvalidTokenId)?
            .supply();
        Ok(supply)
    }
}

impl From<TokenStateError> for Cis2StateError {
    fn from(value: TokenStateError) -> Self {
        match value {
            TokenStateError::InvalidTokenId => Cis2StateError::InvalidTokenId,
        }
    }
}

impl From<HolderStateError> for Cis2StateError {
    fn from(value: HolderStateError) -> Self {
        match value {
            HolderStateError::AmountTooLarge => Cis2StateError::InsufficientFunds,
        }
    }
}
