use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_std::{bail, ensure, Deletable, HasStateApi, Serialize, StateMap};

/// An error that can occur when interacting with the deposited state.
pub enum DepositedStateError {
    /// The token with the given ID was not found.
    TokenNotFound,
    /// The token with the given ID does not have enough deposits.
    InsufficientDeposits,
    /// The token with the given ID does not have enough locked deposits.
    InsufficientLocked,
}

#[derive(Serialize, Clone)]
pub struct DepositedTokenState<A> {
    /// Total Amount Deposited
    pub amount:        A,
    /// Amount Locked. This should be less than or equal to `amount`
    pub locked_amount: A,
}

pub trait IDepositedTokensState<T: Serialize + Clone, A: IsTokenAmount, S: HasStateApi> {
    /// Returns the map of token IDs to their deposited state.
    fn tokens(&self) -> &StateMap<T, DepositedTokenState<A>, S>;

    /// Returns the mutable map of token IDs to their deposited state.
    fn tokens_mut(&mut self) -> &mut StateMap<T, DepositedTokenState<A>, S>;

    /// Returns `true` if the token with the given ID has been deposited.
    fn is_deposited(&self, token_id: &T) -> bool { self.tokens().get(token_id).is_some() }

    /// Returns the total amount of the token with the given ID that has been
    /// deposited.
    fn balance_of_deposited(&self, token_id: &T) -> A {
        self.tokens().get(token_id).map(|info| info.amount).unwrap_or(A::zero())
    }

    /// Returns the total amount of the token with the given ID that has been
    /// locked.
    fn balance_of_locked(&self, token_id: &T) -> A {
        self.tokens().get(token_id).map(|info| info.locked_amount).unwrap_or(A::zero())
    }

    /// Returns the total amount of the token with the given ID that is
    /// unlocked.
    fn balance_of_unlocked(&self, token_id: &T) -> A {
        self.balance_of_deposited(token_id).sub(self.balance_of_locked(token_id))
    }

    /// Adds an entry for the token with the given ID if it does not exist and
    /// sets the balance to the given amount.
    fn inc_deposits(&mut self, token_id: T, amount: A) {
        self.tokens_mut()
            .entry(token_id)
            .and_modify(|into| {
                into.amount.add_assign(amount);
            })
            .or_insert(DepositedTokenState {
                amount,
                locked_amount: A::zero(),
            });
    }

    /// Sets the locked amount for the token with the given ID.
    fn set_locked_deposits(
        &mut self,
        token_id: &T,
        locked_amount: A,
    ) -> Result<(), DepositedStateError> {
        let token = self.tokens_mut().get_mut(token_id);
        match token {
            Some(mut token) => {
                ensure!(token.amount.ge(&locked_amount), DepositedStateError::InsufficientDeposits);
                token.locked_amount = locked_amount;
            }
            None => bail!(DepositedStateError::TokenNotFound),
        };

        Ok(())
    }

    /// Increases the locked amount for the token with the given ID.
    fn inc_locked_deposits(&mut self, token_id: &T, amount: A) -> Result<(), DepositedStateError> {
        let token = self.tokens_mut().get_mut(token_id);
        match token {
            Some(mut token) => {
                token.locked_amount.add_assign(amount);
                ensure!(
                    token.amount.ge(&token.locked_amount),
                    DepositedStateError::InsufficientDeposits
                );
            }
            None => bail!(DepositedStateError::TokenNotFound),
        };

        Ok(())
    }

    /// Decreases the balance of the token with the given ID and removes it if
    /// the balance becomes zero.
    /// Returns `true` if the token was removed.
    /// Returns `false` if the token was not removed.
    fn dec_deposits(&mut self, token_id: &T, amount: A) -> Result<bool, DepositedStateError> {
        let token = self.tokens_mut().get_mut(token_id);
        let token = match token {
            Some(mut token) => {
                token.amount.sub_assign(amount);
                ensure!(
                    token.amount.ge(&token.locked_amount),
                    DepositedStateError::InsufficientDeposits
                );

                token
            }
            None => bail!(DepositedStateError::TokenNotFound),
        };

        if token.amount.is_zero() {
            token.clone().delete();
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Decreases the locked amount for the token with the given ID.
    fn dec_locked_deposits(
        &mut self,
        token_id: &T,
        delta_locked_amount: A,
    ) -> Result<A, DepositedStateError> {
        let token = self.tokens_mut().get_mut(token_id);
        let token = match token {
            Some(mut token) => {
                ensure!(
                    token.locked_amount.ge(&delta_locked_amount),
                    DepositedStateError::InsufficientLocked
                );
                token.locked_amount.sub_assign(delta_locked_amount);
                token
            }
            None => bail!(DepositedStateError::TokenNotFound),
        };

        Ok(token.locked_amount)
    }

    /// Burns the specified amount of tokens. ie Removed them from the total
    /// deposited amount and from the locked amount
    fn burn_locked_deposits(
        &mut self,
        token_id: &T,
        delta_amount: A,
    ) -> Result<A, DepositedStateError> {
        let token = self.tokens_mut().get_mut(token_id);
        let token = match token {
            Some(mut token) => {
                ensure!(
                    token.locked_amount.ge(&delta_amount),
                    DepositedStateError::InsufficientLocked
                );
                token.locked_amount.sub_assign(delta_amount);
                token.amount.sub_assign(delta_amount);
                token
            }
            None => bail!(DepositedStateError::TokenNotFound),
        };

        Ok(token.locked_amount)
    }
}
