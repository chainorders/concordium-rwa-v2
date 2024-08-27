use concordium_std::{Address, ContractAddress, HasStateApi, StateSet};

pub type Sponsor = ContractAddress;

/// Trait for managing sponsors in a state.
pub trait ISponsorsState<S: HasStateApi> {
    /// Returns a reference to the set of sponsors.
    ///
    /// # Returns
    ///
    /// A reference to the set of sponsors.
    fn sponsors(&self) -> &StateSet<Sponsor, S>;

    /// Returns a mutable reference to the set of sponsors.
    ///
    /// # Returns
    ///
    /// A mutable reference to the set of sponsors.
    fn sponsors_mut(&mut self) -> &mut StateSet<Sponsor, S>;

    /// Checks if the given address is a sponsor.
    ///
    /// # Arguments
    ///
    /// * `sponsor` - An address to check.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the address is a sponsor.
    fn is_sponsor(&self, sponsor: &Address) -> bool {
        match sponsor {
            Address::Contract(sponsor) => self.sponsors().contains(sponsor),
            _ => false,
        }
    }

    /// Adds the given address to the set of sponsors.
    ///
    /// # Arguments
    ///
    /// * `sponsor` - An address to add.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the address was added successfully.
    fn add_sponsor(&mut self, sponsor: Sponsor) -> bool { self.sponsors_mut().insert(sponsor) }

    /// Removes the given address from the set of sponsors.
    ///
    /// # Arguments
    ///
    /// * `sponsor` - An address to remove.
    ///
    /// # Returns
    ///
    /// A boolean indicating whether the address was removed successfully.
    fn remove_sponsor(&mut self, sponsor: &Sponsor) -> bool { self.sponsors_mut().remove(sponsor) }

    /// Returns a list of all sponsors.
    ///
    /// # Returns
    ///
    /// A vector containing all sponsors.
    fn list_sponsors(&self) -> Vec<Sponsor> { self.sponsors().iter().map(|a| *a).collect() }
}
