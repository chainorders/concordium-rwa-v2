use concordium_protocols::concordium_cis2_ext::{IsTokenAmount, IsTokenId};
use concordium_std::*;

pub enum HolderStateError {
    AmountTooLarge,
}
pub type HolderStateResult<T> = Result<T, HolderStateError>;

pub trait IHolderState<T, A, S: HasStateApi>: Serial+DeserialWithState<S> {
    fn new(state_builder: &mut StateBuilder<S>) -> Self;
    fn is_operator(&self, operator: &Address) -> bool;
    fn add_operator(&mut self, operator: Address);
    fn remove_operator(&mut self, operator: &Address);
    fn balance_of(&self, token_id: &T) -> A;
    fn sub_balance(&mut self, token_id: &T, amount: &A);
    fn add_balance(&mut self, token_id: &T, amount: &A);
}

pub trait IHoldersState<
    T: IsTokenId,
    A: IsTokenAmount,
    THolderState: IHolderState<T, A, S>,
    S: HasStateApi,
> {
    fn holders(&self) -> &StateMap<Address, THolderState, S>;
    fn holders_mut(&mut self) -> &mut StateMap<Address, THolderState, S>;

    /// Checks if the given operator is an operator for the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to check.
    /// * `operator` - The operator to check.
    ///
    /// # Returns
    ///
    /// Returns `true` if the operator is an operator for the address, `false`
    /// otherwise.
    fn is_operator(&self, address: &Address, operator: &Address) -> bool {
        self.holders()
            .get(address)
            .map(|address| address.is_operator(operator))
            .unwrap_or(false)
    }

    /// Adds an operator for the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to add an operator for.
    /// * `operator` - The operator to add.
    /// * `state_builder` - A mutable reference to the state builder.
    fn add_operator(
        &mut self,
        address: Address,
        operator: Address,
        state_builder: &mut StateBuilder<S>,
    ) {
        self.holders_mut()
            .entry(address)
            .or_insert_with(|| THolderState::new(state_builder))
            .add_operator(operator);
    }

    /// Removes an operator from the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to remove an operator from.
    /// * `operator` - The operator to remove.
    fn remove_operator(&mut self, address: Address, operator: &Address) {
        self.holders_mut().entry(address).and_modify(|address| {
            address.remove_operator(operator);
        });
    }

    /// Returns the balance of the given address for the given token.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to get the balance for.
    /// * `token_id` - The ID of the token to get the balance for.
    ///
    /// # Returns
    ///
    /// Returns the balance of the given token for the given address. If the
    /// address does not exist, returns zero.
    fn balance_of(&self, address: &Address, token_id: &T) -> A {
        self.holders()
            .get(address)
            .map(|address| address.balance_of(token_id))
            .unwrap_or(A::zero())
    }

    fn ensure_has_sufficient_balance(
        &self,
        address: &Address,
        token_id: &T,
        amount: &A,
    ) -> HolderStateResult<()> {
        let balance = self.balance_of(address, token_id);
        if balance.lt(amount) {
            Err(HolderStateError::AmountTooLarge)
        } else {
            Ok(())
        }
    }

    /// Adds a specified amount to the balance of a specific token for the given
    /// address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to add the balance to.
    /// * `token_id` - The ID of the token to add the balance to.
    /// * `amount_delta` - The amount to add.
    /// * `state_builder` - A mutable reference to the state builder.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation was successful. If the address does
    /// not exist, it is created.
    ///
    /// # Errors
    ///
    /// Returns `HolderStateError::AmountOverflow` if the amount to add and the
    /// current balance exceed the maximum value of `A`.
    fn add_balance(
        &mut self,
        address: &Address,
        token_id: &T,
        amount_delta: &A,
        state_builder: &mut StateBuilder<S>,
    ) {
        self.holders_mut()
            .entry(*address)
            .or_insert_with(|| IHolderState::new(state_builder))
            .modify(|holder| holder.add_balance(token_id, amount_delta));
    }

    /// Subtracts a specified amount from the balance of a specific token for
    /// the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to subtract the balance from.
    /// * `token_id` - The ID of the token to subtract the balance from.
    /// * `amount_delta` - The amount to subtract.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the operation was successful.
    ///
    /// # Errors
    ///
    /// Returns `HolderStateError::AmountTooLarge` if the amount to subtract is
    /// larger than the current balance.
    fn sub_balance(
        &mut self,
        address: &Address,
        token_id: &T,
        amount_delta: &A,
    ) -> HolderStateResult<()> {
        self.holders_mut()
            .entry(*address)
            .occupied_or(HolderStateError::AmountTooLarge)?
            .modify(|holder| holder.sub_balance(token_id, amount_delta));

        Ok(())
    }
}
