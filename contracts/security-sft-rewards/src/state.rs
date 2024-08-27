use concordium_cis2::{IsTokenId, TokenAmountU64, TokenIdVec};
use concordium_protocols::concordium_cis2_ext::{IsTokenAmount, PlusSubOne};
use concordium_rwa_utils::conversions::exchange_rate::Rate;
use concordium_std::ops::{Add, AddAssign, Sub, SubAssign};
use concordium_std::{
    ensure, Address, ContractAddress, Deletable, Deserial, DeserialWithState, HasStateApi,
    MetadataUrl, Serial, Serialize, StateApi, StateBuilder, StateMap, StateRef, StateRefMut,
    StateSet,
};

use super::types::{AgentRole, TokenAmount, TokenId};
use crate::error::Error;

#[derive(Serialize, Clone)]
pub struct RewardDeposited<TDeposit: IsTokenId, ADeposit: IsTokenAmount> {
    pub token_id:       TDeposit,
    pub token_contract: ContractAddress,
    pub token_amount:   ADeposit,
    pub rate:           Rate,
}

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
    pub tokens:            StateMap<TokenId, TokenState, S>,
    pub identity_registry: ContractAddress,
    pub compliance:        ContractAddress,
    pub addresses:         StateMap<Address, AddressState<S>, S>,
    pub rewards_ids_range: (TokenId, TokenId),
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

    pub fn token(&self, token_id: &TokenId) -> Option<StateRef<TokenState>> {
        self.tokens.get(token_id)
    }

    pub fn token_mut(&mut self, token_id: &TokenId) -> Option<StateRefMut<TokenState, S>> {
        self.tokens.get_mut(token_id)
    }

    pub fn add_token(&mut self, token_id: TokenId, token_state: TokenState) -> Result<(), Error> {
        self.tokens
            .entry(token_id)
            .vacant_or(Error::InvalidTokenId)?
            .insert(token_state);
        Ok(())
    }

    pub fn sub_assign_supply(
        &mut self,
        token_id: &TokenId,
        amount: TokenAmount,
    ) -> Result<(), Error> {
        self.token_mut(token_id)
            .ok_or(Error::InvalidTokenId)?
            .main_mut()
            .ok_or(Error::InvalidTokenId)?
            .sub_assign_supply(amount)?;
        Ok(())
    }

    pub fn sub_assign_supply_rewards(
        &mut self,
        rewards: &Vec<(TokenId, TokenAmount)>,
    ) -> Result<(), Error> {
        for (token_id, amount) in rewards {
            self.token_mut(token_id)
                .ok_or(Error::InvalidRewardTokenId)?
                .reward_mut()
                .ok_or(Error::InvalidRewardTokenId)?
                .sub_assign_supply(*amount)?;
        }
        Ok(())
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

#[derive(Serialize, Clone)]
pub struct RewardTokenState {
    pub reward:       Option<RewardDeposited<TokenIdVec, TokenAmountU64>>,
    pub metadata_url: MetadataUrl,
    pub supply:       TokenAmount,
}

impl RewardTokenState {
    pub fn sub_assign_supply(&mut self, amount: TokenAmount) -> Result<TokenAmount, Error> {
        ensure!(self.supply.ge(&amount), Error::InsufficientFunds);
        self.supply.sub_assign(amount);
        Ok(self.supply)
    }

    pub fn add_assign_supply(&mut self, amount: TokenAmount) { self.supply.add_assign(amount); }

    pub fn metadata_url(&self) -> &MetadataUrl { &self.metadata_url }

    pub fn attach_reward(
        &mut self,
        metadata_url: MetadataUrl,
        reward: RewardDeposited<TokenIdVec, TokenAmountU64>,
    ) {
        self.reward = Some(reward);
        self.metadata_url = metadata_url;
    }
}

#[derive(Serialize, Clone)]
pub enum TokenState {
    Main(MainTokenState),
    Reward(RewardTokenState),
}

impl TokenState {
    pub fn main(&self) -> Option<&MainTokenState> {
        match self {
            TokenState::Main(main) => Some(main),
            _ => None,
        }
    }

    pub fn main_mut(&mut self) -> Option<&mut MainTokenState> {
        match self {
            TokenState::Main(main) => Some(main),
            _ => None,
        }
    }

    pub fn reward(&self) -> Option<&RewardTokenState> {
        match self {
            TokenState::Reward(reward) => Some(reward),
            _ => None,
        }
    }

    pub fn reward_mut(&mut self) -> Option<&mut RewardTokenState> {
        match self {
            TokenState::Reward(reward) => Some(reward),
            _ => None,
        }
    }

    pub fn metadata_url(&self) -> &MetadataUrl {
        match self {
            TokenState::Main(main) => main.metadata_url(),
            TokenState::Reward(reward) => reward.metadata_url(),
        }
    }

    pub fn add_assign_supply(&mut self, amount: TokenAmount) -> Result<(), Error> {
        match self {
            TokenState::Main(main) => main.add_assign_supply(amount),
            TokenState::Reward(reward) => {
                reward.add_assign_supply(amount);
                Ok(())
            }
        }
    }
}

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

    pub fn sub_assign_balance_rewards(
        &mut self,
        reward_token_range: &(TokenId, TokenId),
        total_amount: TokenAmount,
    ) -> Result<Vec<(TokenId, TokenAmount)>, Error> {
        let mut res = vec![];
        let (min_reward_token_id, max_reward_token_id) = reward_token_range;
        let mut total_reward_amount = total_amount;
        let mut token_id = *min_reward_token_id;

        while total_reward_amount.gt(&TokenAmount::zero()) && token_id.le(max_reward_token_id) {
            if let Some(mut holder_balance) = self.balance_mut(&token_id) {
                let rewarded_amount: TokenAmount = holder_balance.min(total_reward_amount);
                if rewarded_amount.eq(&TokenAmount::zero()) {
                    continue;
                }
                holder_balance.sub_assign(rewarded_amount)?;
                total_reward_amount.sub_assign(rewarded_amount);
                res.push((token_id, rewarded_amount));
            }

            token_id.plus_one_assign();
        }
        ensure!(
            total_reward_amount.eq(&TokenAmount::zero()),
            Error::InsufficientRewardFunds
        );

        Ok(res)
    }

    pub fn add_assign_balance_rewards(
        &mut self,
        rewards: &Vec<(TokenId, TokenAmount)>,
    ) -> Result<(), Error> {
        for (token_id, amount) in rewards {
            self.balances
                .entry(*token_id)
                .or_insert(HolderStateBalance::default())
                .modify(|balance| balance.add_assign(*amount));
        }
        Ok(())
    }

    pub fn clone_for_recovery(&self, state_builder: &mut StateBuilder<S>) -> Self {
        let mut new_holder = HolderState::new(state_builder);
        for (token_id, balance) in self.balances.iter() {
            let _ = new_holder.balances.insert(*token_id, balance.clone());
        }
        new_holder
    }
}
