use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_std::ops::{Add, AddAssign, Sub, SubAssign};
use concordium_std::{
    ensure, Address, ContractAddress, Deletable, Deserial, DeserialWithState, HasStateApi,
    MetadataUrl, Serial, Serialize, StateApi, StateBuilder, StateMap, StateRef, StateRefMut,
    StateSet,
};

use super::types::{AgentRole, TokenAmount, TokenId};
use crate::error::Error;

#[derive(Serial, DeserialWithState, Deletable)]
#[concordium(state_parameter = "S")]
pub enum HolderAddressState<S> {
    Holder(HolderState<S>),
    Recovered(Address),
}

impl<S: HasStateApi> HolderAddressState<S> {
    pub fn active(&self) -> Option<&HolderState<S>> {
        match self {
            HolderAddressState::Holder(holder) => Some(holder),
            _ => None,
        }
    }

    pub fn active_mut(&mut self) -> Option<&mut HolderState<S>> {
        match self {
            HolderAddressState::Holder(holder) => Some(holder),
            _ => None,
        }
    }

    pub fn recovered(&self) -> Option<&Address> {
        match self {
            HolderAddressState::Recovered(address) => Some(address),
            _ => None,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct AgentState(pub Vec<AgentRole>);
impl AgentState {
    pub fn has_roles(&self, roles: &[AgentRole]) -> bool {
        roles.iter().all(|r| self.0.contains(r))
    }

    pub fn roles(&self) -> &Vec<AgentRole> { &self.0 }
}

#[derive(Serial, DeserialWithState, Deletable)]
#[concordium(state_parameter = "S")]
pub enum AddressState<S> {
    Agent(AgentState),
    Holder(HolderAddressState<S>),
}

impl<S> AddressState<S> {
    pub fn agent(&self) -> Option<&AgentState> {
        match self {
            AddressState::Agent(agent) => Some(agent),
            _ => None,
        }
    }

    pub fn is_agent(&self, roles: &[AgentRole]) -> bool {
        match self {
            AddressState::Agent(agent) => agent.has_roles(roles),
            _ => false,
        }
    }

    pub fn holder(&self) -> Option<&HolderAddressState<S>> {
        match self {
            AddressState::Holder(holder) => Some(holder),
            _ => None,
        }
    }

    pub fn holder_mut(&mut self) -> Option<&mut HolderAddressState<S>> {
        match self {
            AddressState::Holder(holder) => Some(holder),
            _ => None,
        }
    }
}

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
/// Represents the state of the security NFT contract.
pub struct State<S=StateApi> {
    pub token:             TokenState,
    pub identity_registry: ContractAddress,
    pub compliance:        ContractAddress,
    pub addresses:         StateMap<Address, AddressState<S>, S>,
    pub sponsor:           Option<ContractAddress>,
}

impl<S: HasStateApi> State<S> {
    pub fn address(&self, address: &Address) -> Option<StateRef<AddressState<S>>> {
        self.addresses.get(address)
    }

    pub fn address_or_insert_holder(
        &mut self,
        address: &Address,
        state_builder: &mut StateBuilder<S>,
    ) -> StateRefMut<AddressState<S>, S> {
        self.addresses
            .entry(*address)
            .or_insert(AddressState::Holder(HolderAddressState::Holder(
                HolderState::new(state_builder),
            )));
        self.address_mut(address).unwrap()
    }

    pub fn address_mut(&mut self, address: &Address) -> Option<StateRefMut<AddressState<S>, S>> {
        self.addresses.get_mut(address)
    }

    pub fn add_address(
        &mut self,
        address: Address,
        state: AddressState<S>,
    ) -> Result<StateRefMut<AddressState<S>, S>, Error> {
        self.addresses
            .entry(address)
            .vacant_or(Error::InvalidAddress)?
            .insert(state);
        let address = self.address_mut(&address).ok_or(Error::InvalidAddress)?;
        Ok(address)
    }

    pub fn remove_and_get_address(&mut self, address: &Address) -> Result<AddressState<S>, Error> {
        let address = self
            .addresses
            .remove_and_get(address)
            .ok_or(Error::InvalidAddress)?;
        Ok(address)
    }

    pub fn sub_assign_supply(&mut self, _: &TokenId, amount: TokenAmount) -> Result<(), Error> {
        self.token.sub_assign_supply(amount)?;
        Ok(())
    }

    pub fn add_assign_supply(&mut self, _: &TokenId, amount: TokenAmount) -> Result<(), Error> {
        self.token.add_assign_supply(amount)
    }
}

#[derive(Serialize, Clone)]
pub struct MainTokenState {
    pub metadata_url: MetadataUrl,
    pub paused:       bool,
    pub supply:       TokenAmount,
}

impl MainTokenState {
    pub fn pause(&mut self) { self.paused = true; }

    pub fn un_pause(&mut self) { self.paused = false; }

    pub fn sub_assign_supply(&mut self, amount: TokenAmount) -> Result<TokenAmount, Error> {
        ensure!(!self.paused, Error::PausedToken);
        ensure!(self.supply.ge(&amount), Error::InsufficientFunds);
        self.supply.sub_assign(amount);
        Ok(self.supply)
    }

    pub fn add_assign_supply(&mut self, amount: TokenAmount) -> Result<(), Error> {
        ensure!(!self.paused, Error::PausedToken);
        self.supply.add_assign(amount);
        Ok(())
    }

    pub fn metadata_url(&self) -> &MetadataUrl { &self.metadata_url }
}

pub type TokenState = MainTokenState;

#[derive(Deserial, Serial, Clone)]
pub struct HolderStateBalance {
    pub frozen:    TokenAmount,
    pub un_frozen: TokenAmount,
}
impl HolderStateBalance {
    pub fn default() -> Self {
        Self {
            frozen:    TokenAmount::zero(),
            un_frozen: TokenAmount::zero(),
        }
    }

    pub fn total(&self) -> TokenAmount { self.frozen.add(self.un_frozen) }

    pub fn sub_assign(&mut self, amount: TokenAmount) -> Result<TokenAmount, Error> {
        ensure!(self.un_frozen.ge(&amount), Error::InsufficientFunds);
        self.un_frozen.sub_assign(amount);
        Ok(self.un_frozen)
    }

    pub fn add_assign(&mut self, amount: TokenAmount) { self.un_frozen.add_assign(amount); }

    pub fn min(&self, amount: TokenAmount) -> TokenAmount { self.un_frozen.min(amount) }

    pub fn freeze(&mut self, amount: TokenAmount) -> Result<(), Error> {
        ensure!(self.un_frozen.ge(&amount), Error::InsufficientFunds);
        self.frozen.add_assign(amount);
        self.un_frozen.sub_assign(amount);

        Ok(())
    }

    pub fn un_freeze(&mut self, amount: TokenAmount) -> Result<(), Error> {
        ensure!(self.frozen.ge(&amount), Error::InsufficientFunds);
        self.frozen.sub_assign(amount);
        self.un_frozen.add_assign(amount);

        Ok(())
    }
}

#[derive(DeserialWithState, Serial, Deletable)]
#[concordium(state_parameter = "S")]
pub struct HolderState<S=StateApi> {
    pub operators: StateSet<Address, S>,
    pub balances:  StateMap<TokenId, HolderStateBalance, S>,
}
impl<S: HasStateApi> HolderState<S> {
    pub fn new(state_builder: &mut StateBuilder<S>) -> Self {
        HolderState {
            operators: state_builder.new_set(),
            balances:  state_builder.new_map(),
        }
    }

    pub fn balance(&self, token_id: &TokenId) -> Option<StateRef<HolderStateBalance>> {
        self.balances.get(token_id)
    }

    pub fn balance_mut(
        &mut self,
        token_id: &TokenId,
    ) -> Option<StateRefMut<HolderStateBalance, S>> {
        self.balances.get_mut(token_id)
    }

    pub fn has_operator(&self, operator: &Address) -> bool { self.operators.contains(operator) }

    pub fn add_operator(&mut self, operator: Address) { self.operators.insert(operator); }

    pub fn remove_operator(&mut self, operator: &Address) { self.operators.remove(operator); }

    pub fn un_freeze_balance_to_match(
        &mut self,
        token_id: &TokenId,
        amount: TokenAmount,
    ) -> Result<TokenAmount, Error> {
        let mut holder_balance = self.balance_mut(token_id).ok_or(Error::InsufficientFunds)?;
        if holder_balance.un_frozen.ge(&amount) {
            return Ok(TokenAmount::zero());
        }

        let un_frozen_amount = amount.sub(holder_balance.un_frozen);
        if un_frozen_amount.gt(&holder_balance.frozen) {
            return Err(Error::InsufficientFunds);
        }

        holder_balance.frozen.sub_assign(un_frozen_amount);
        holder_balance.un_frozen.add_assign(un_frozen_amount);
        Ok(un_frozen_amount)
    }

    pub fn sub_assign_balance(
        &mut self,
        token_id: &TokenId,
        amount: TokenAmount,
    ) -> Result<TokenAmount, Error> {
        self.balances
            .entry(*token_id)
            .occupied_or(Error::InsufficientFunds)?
            .sub_assign(amount)
    }

    pub fn add_assign_balance(&mut self, token_id: &TokenId, amount: TokenAmount) {
        self.balances
            .entry(*token_id)
            .or_insert(HolderStateBalance::default())
            .modify(|balance| balance.add_assign(amount));
    }

    pub fn clone_for_recovery(&self, state_builder: &mut StateBuilder<S>) -> Self {
        let mut new_holder = HolderState::new(state_builder);
        for (token_id, balance) in self.balances.iter() {
            let _ = new_holder.balances.insert(*token_id, balance.clone());
        }
        new_holder
    }
}
