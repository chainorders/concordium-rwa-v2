use concordium_protocols::concordium_cis2_ext::{IsTokenAmount, IsTokenId};
use concordium_std::*;

use super::holders_state::{HolderStateError, IHolderState, IHoldersState};

/// Represents the different types of errors that can occur in the holder's
/// security state.
pub enum HolderSecurityStateError {
    /// Triggered when the amount is too large for the operation.
    AmountTooLarge,
    /// Triggered when the address has already been recovered.
    AddressAlreadyRecovered,
    /// Triggered when the recovery address provided is invalid.
    InvalidRecoveryAddress,
}

impl From<HolderStateError> for HolderSecurityStateError {
    fn from(e: HolderStateError) -> Self {
        match e {
            HolderStateError::AmountTooLarge => HolderSecurityStateError::AmountTooLarge,
        }
    }
}

pub type HolderSecurityStateResult<T> = Result<T, HolderSecurityStateError>;

pub trait ISecurityHolderState<T, A, S> {
    fn freeze(&mut self, token_id: &T, amount: &A) -> HolderSecurityStateResult<()>;
    fn un_freeze(&mut self, token_id: &T, amount: &A) -> HolderSecurityStateResult<()>;
    fn balance_of_frozen(&self, token_id: &T) -> A;
    fn balance_of_un_frozen(&self, token_id: &T) -> A;
}

pub trait IHoldersSecurityState<
    T: IsTokenId,
    A: IsTokenAmount,
    THolderState: IHolderState<T, A, S>+ISecurityHolderState<T, A, S>,
    S: HasStateApi,
>: IHoldersState<T, A, THolderState, S> {
    fn recovery_addresses(&self) -> &StateMap<Address, Address, S>;
    fn recovery_addresses_mut(&mut self) -> &mut StateMap<Address, Address, S>;

    /// Returns the recovery address for the given address if it exists.
    /// Retrieves the recovery address associated with the given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address for which to retrieve the recovery address.
    ///
    /// # Returns
    ///
    /// An `Option<Address>` representing the recovery address, or `None` if no
    /// recovery address is found.
    fn get_recovery_address(&self, address: &Address) -> Option<Address> {
        self.recovery_addresses().get(address).map(|a| *a)
    }

    /// Ensures that the given address has not been recovered.
    ///
    /// # Errors
    ///
    /// Returns a `RecoveryError::AddressAlreadyRecovered` error if the address
    /// has already been recovered.
    fn ensure_not_recovered(&self, address: &Address) -> HolderSecurityStateResult<()> {
        ensure!(
            self.get_recovery_address(address).is_none(),
            HolderSecurityStateError::AddressAlreadyRecovered
        );

        Ok(())
    }

    /// Sets the recovery address for the given address.
    /// Sets the recovery address for a given address in the holders security
    /// state.
    ///
    /// # Arguments
    ///
    /// * `address` - The address for which the recovery address is being set.
    /// * `recovery_address` - The recovery address to be set.
    fn set_recovery_address(&mut self, address: Address, recovery_address: Address) {
        self.recovery_addresses_mut()
            .entry(address)
            .or_insert_with(|| recovery_address);
    }

    /// Removes the recovery address of the given address and sets it to the new
    /// address. Also transfers any frozen balances from the old address to
    /// the new address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to remove the recovery address from.
    /// * `new_address` - The new address to set as the recovery address.
    ///
    /// # Errors
    ///
    /// Returns `RecoveryError::AddressAlreadyRecovered` if the address has
    /// already been recovered.
    /// Returns `RecoveryError::InvalidRecoveryAddress` if the new address
    /// already has state / frozen balances / already been recovered.
    fn recover(&mut self, address: Address, new_address: Address) -> HolderSecurityStateResult<()> {
        let old_state = self.holders_mut().remove_and_get(&address);
        match old_state {
            Some(old_state) => match self.holders_mut().insert(new_address, old_state) {
                None => match self.recovery_addresses_mut().insert(address, new_address) {
                    None => Ok(()),
                    Some(_) => Err(HolderSecurityStateError::AddressAlreadyRecovered),
                },
                Some(_) => Err(HolderSecurityStateError::InvalidRecoveryAddress),
            },
            None => Ok(()),
        }
    }

    /// Adds a specified amount to the frozen balance of a given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to freeze the balance for.
    /// * `token_id` - The ID of the token to freeze.
    /// * `amount` - The amount to freeze.
    /// * `state_builder` - A mutable reference to the state builder.
    fn freeze(
        &mut self,
        address: Address,
        token_id: &T,
        amount: &A,
    ) -> HolderSecurityStateResult<()> {
        self.holders_mut()
            .entry(address)
            .occupied_or(HolderSecurityStateError::AmountTooLarge)?
            .modify(|h| h.freeze(token_id, amount))
    }

    /// Subtracts a specified amount from the frozen balance of a given address.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to unfreeze the balance for.
    /// * `token_id` - The ID of the token to unfreeze.
    /// * `amount` - The amount to unfreeze.
    fn un_freeze(
        &mut self,
        address: &Address,
        token_id: &T,
        amount: &A,
    ) -> HolderSecurityStateResult<()> {
        self.holders_mut()
            .entry(*address)
            .occupied_or(HolderSecurityStateError::AmountTooLarge)?
            .modify(|h| h.un_freeze(token_id, amount))
    }

    /// Returns the frozen balance of a given address for a specific token.
    ///
    /// # Arguments
    ///
    /// * `address` - The address to get the frozen balance for.
    /// * `token_id` - The ID of the token to get the frozen balance for.
    fn balance_of_frozen(&self, address: &Address, token_id: &T) -> A {
        self.holders()
            .get(address)
            .map(|h| h.balance_of_frozen(token_id))
            .unwrap_or(A::zero())
    }

    /// Returns the unfrozen balance of the given token for the given addresses.
    ///
    /// # Parameters
    ///
    /// * `address`: The address to check.
    /// * `token_id`: The token ID to check.
    ///
    /// # Returns
    ///
    /// Returns `TokenAmount` containing the unfrozen balance.
    fn balance_of_unfrozen(&self, address: &Address, token_id: &T) -> A {
        self.holders()
            .get(address)
            .map(|h| h.balance_of_un_frozen(token_id))
            .unwrap_or(A::zero())
    }
}
