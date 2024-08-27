use concordium_cis2::{TokenAmountU64, TokenIdVec};
use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_rwa_utils::state_implementations::cis2_security_state::ICis2SecurityState;
use concordium_rwa_utils::state_implementations::cis2_state::{ICis2State, ICis2TokenState};
use concordium_rwa_utils::state_implementations::holders_security_state::{
    HolderSecurityStateError, HolderSecurityStateResult, IHoldersSecurityState,
    ISecurityHolderState,
};
use concordium_rwa_utils::state_implementations::holders_state::{IHolderState, IHoldersState};
use concordium_rwa_utils::state_implementations::rewards_state::{
    AddRewardParam, IRewardHolderState, IRewardTokenState, IRewardsState, RewardDeposited,
};
use concordium_rwa_utils::state_implementations::sponsors_state::ISponsorsState;
use concordium_rwa_utils::state_implementations::tokens_security_state::{
    ISecurityTokenState, ITokensSecurityState,
};
use concordium_rwa_utils::state_implementations::tokens_state::{ITokenState, ITokensState};
use concordium_std::ops::{Add, Sub, SubAssign};
use concordium_std::{
    Address, ContractAddress, Deserial, DeserialWithState, HasStateApi, MetadataUrl, Serial,
    Serialize, StateApi, StateBuilder, StateMap, StateSet,
};

use super::types::{Agent, AgentRole, TokenAmount, TokenId};

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
/// Represents the state of the security NFT contract.
pub struct State<S=StateApi> {
    pub tracked_token_id: TokenId,
    /// A set that stores the addresses of the agents in the contract.
    agents: StateMap<Address, StateSet<AgentRole, S>, S>,
    /// A map that stores the state of each token in the contract.
    tokens: StateMap<TokenId, TokenState, S>,
    /// A map that stores the state of each holder's address for each token.
    holders: StateMap<Address, HolderState, S>,
    /// A set that stores the addresses of the sponsors of the contract.
    sponsors: StateSet<ContractAddress, S>,
    pub identity_registry: ContractAddress,
    pub compliance: ContractAddress,
    recovery_addresses: StateMap<Address, Address, S>,
    // rewards
    pub max_reward_token_id: TokenId,
    pub min_reward_token_id: TokenId,
    pub blank_reward_metadata_url: MetadataUrl,
}

impl<S: HasStateApi> State<S> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identity_registry: ContractAddress,
        compliance: ContractAddress,
        sponsors: Vec<ContractAddress>,
        agents: Vec<Agent>,
        metadata_url: MetadataUrl,
        blank_reward_metadata_url: MetadataUrl,
        traced_token_id: TokenId,
        min_reward_token_id: TokenId,
        state_builder: &mut StateBuilder<S>,
    ) -> Self {
        let mut state_tokens = state_builder.new_map();

        // Insert Tokens
        let _ = state_tokens.insert(traced_token_id, TokenState::new(metadata_url));
        let _ = state_tokens.insert(
            min_reward_token_id,
            TokenState::new(blank_reward_metadata_url.clone()),
        );

        // Insert Agents
        let mut state_agents = state_builder.new_map();
        for agent in agents {
            let mut agent_roles = state_builder.new_set();
            agent.roles.iter().for_each(|r| {
                agent_roles.insert(*r);
            });
            let _ = state_agents.insert(agent.address, agent_roles);
        }

        // Insert Sponsors
        let mut state_sponsors = state_builder.new_set();
        sponsors.iter().for_each(|s| {
            let _ = state_sponsors.insert(*s);
        });

        State {
            tracked_token_id: traced_token_id,
            agents: state_agents,
            tokens: state_tokens,
            holders: state_builder.new_map(),
            sponsors: state_sponsors,
            identity_registry,
            compliance,
            recovery_addresses: state_builder.new_map(),
            max_reward_token_id: min_reward_token_id,
            min_reward_token_id,
            blank_reward_metadata_url,
        }
    }
}

impl IAgentWithRolesState<StateApi, AgentRole> for State {
    fn agents(&self) -> &StateMap<Address, StateSet<AgentRole, StateApi>, StateApi> { &self.agents }

    fn agents_mut(&mut self) -> &mut StateMap<Address, StateSet<AgentRole, StateApi>, StateApi> {
        &mut self.agents
    }
}
impl ISponsorsState<StateApi> for State {
    fn sponsors(&self) -> &StateSet<ContractAddress, StateApi> { &self.sponsors }

    fn sponsors_mut(&mut self) -> &mut StateSet<ContractAddress, StateApi> { &mut self.sponsors }
}

#[derive(Serialize, Clone)]
pub struct TokenState {
    metadata_url: MetadataUrl,
    paused:       bool,
    supply:       TokenAmount,
    reward:       Option<RewardDeposited<TokenIdVec, TokenAmountU64>>,
}

impl TokenState {
    pub fn new(metadata_url: MetadataUrl) -> Self {
        Self {
            metadata_url,
            paused: false,
            supply: TokenAmount::zero(),
            reward: None,
        }
    }

    pub fn metadata_url(&self) -> &MetadataUrl { &self.metadata_url }
}
impl ITokenState for TokenState {
    fn metadata_url(&self) -> &MetadataUrl { &self.metadata_url }
}
impl ISecurityTokenState for TokenState {
    fn paused(&self) -> bool { self.paused }

    fn set_paused(&mut self, is_paused: bool) { self.paused = is_paused; }
}
impl ICis2TokenState<TokenAmount> for TokenState {
    fn inc_supply(&mut self, amount: &TokenAmount) { self.supply = self.supply.add(*amount); }

    fn dec_supply(&mut self, amount: &TokenAmount) { self.supply = self.supply.sub(*amount); }

    fn supply(&self) -> TokenAmount { self.supply }
}

#[derive(Deserial, Serial)]
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

    pub fn freeze(&mut self, amount: &TokenAmount) {
        self.frozen = self.frozen.add(*amount);
        self.un_frozen = self.un_frozen.sub(*amount);
    }

    pub fn un_freeze(&mut self, amount: &TokenAmount) {
        self.frozen = self.frozen.sub(*amount);
        self.un_frozen = self.un_frozen.add(*amount);
    }

    pub fn add(&mut self, amount: TokenAmount) { self.un_frozen = self.un_frozen.add(amount); }

    pub fn sub(&mut self, amount: TokenAmount) { self.un_frozen = self.un_frozen.sub(amount); }

    pub fn sub_forced(&mut self, amount: TokenAmount) -> TokenAmount {
        self.un_frozen = self.un_frozen.sub(self.un_frozen.min(amount));
        let un_freeze = amount.sub(self.un_frozen);
        self.frozen = self.frozen.sub(un_freeze);

        un_freeze
    }
}

#[derive(DeserialWithState, Serial)]
#[concordium(state_parameter = "S")]
pub struct HolderState<S=StateApi> {
    pub operators: StateSet<Address, S>,
    pub balances:  StateMap<TokenId, HolderStateBalance, S>,
}
impl IHolderState<TokenId, TokenAmount, StateApi> for HolderState {
    fn is_operator(&self, operator: &Address) -> bool { self.operators.contains(operator) }

    fn add_operator(&mut self, operator: Address) { self.operators.insert(operator); }

    fn remove_operator(&mut self, operator: &Address) { self.operators.remove(operator); }

    fn balance_of(&self, token_id: &TokenId) -> TokenAmount {
        self.balances
            .get(token_id)
            .map(|a| a.total())
            .unwrap_or(TokenAmount::zero())
    }

    fn add_balance(&mut self, token_id: &TokenId, amount: &TokenAmount) {
        self.balances
            .entry(*token_id)
            .or_insert(HolderStateBalance::default())
            .modify(|a| a.add(*amount));
    }

    fn sub_balance(&mut self, token_id: &TokenId, amount: &TokenAmount) {
        self.balances
            .entry(*token_id)
            .or_insert(HolderStateBalance::default())
            .modify(|a| a.sub(*amount));
    }

    fn new(state_builder: &mut StateBuilder<StateApi>) -> Self {
        HolderState {
            operators: state_builder.new_set(),
            balances:  state_builder.new_map(),
        }
    }
}
impl ISecurityHolderState<TokenId, TokenAmount, StateApi> for HolderState {
    fn freeze(
        &mut self,
        token_id: &TokenId,
        amount: &TokenAmount,
    ) -> HolderSecurityStateResult<()> {
        self.balances
            .entry(*token_id)
            .occupied_or(HolderSecurityStateError::AmountTooLarge)?
            .modify(|e| {
                e.freeze(amount);
            });
        Ok(())
    }

    fn un_freeze(
        &mut self,
        token_id: &TokenId,
        amount: &TokenAmount,
    ) -> HolderSecurityStateResult<()> {
        self.balances
            .entry(*token_id)
            .occupied_or(HolderSecurityStateError::AmountTooLarge)?
            .modify(|e| e.un_freeze(amount));
        Ok(())
    }

    fn balance_of_frozen(&self, token_id: &TokenId) -> TokenAmount {
        self.balances
            .get(token_id)
            .map(|a| a.frozen)
            .unwrap_or(TokenAmount::zero())
    }

    fn balance_of_un_frozen(&self, token_id: &TokenId) -> TokenAmount {
        self.balances
            .get(token_id)
            .map(|a| a.un_frozen)
            .unwrap_or(TokenAmount::zero())
    }
}

// State Implementations for Security Tokens
impl ITokensState<TokenId, TokenState, StateApi> for State {
    fn tokens(&self) -> &StateMap<TokenId, TokenState, StateApi> { &self.tokens }

    fn tokens_mut(&mut self) -> &mut StateMap<TokenId, TokenState, StateApi> { &mut self.tokens }
}
impl ITokensSecurityState<TokenId, TokenState, StateApi> for State {}
impl IHoldersState<TokenId, TokenAmount, HolderState, StateApi> for State {
    fn holders(&self) -> &StateMap<Address, HolderState, StateApi> { &self.holders }

    fn holders_mut(&mut self) -> &mut StateMap<Address, HolderState, StateApi> { &mut self.holders }
}
impl IHoldersSecurityState<TokenId, TokenAmount, HolderState, StateApi> for State {
    fn recovery_addresses(&self) -> &StateMap<Address, Address, StateApi> {
        &self.recovery_addresses
    }

    fn recovery_addresses_mut(&mut self) -> &mut StateMap<Address, Address, StateApi> {
        &mut self.recovery_addresses
    }
}
impl ICis2State<TokenId, TokenAmount, TokenState, HolderState, StateApi> for State {}
impl ICis2SecurityState<TokenId, TokenAmount, TokenState, HolderState, StateApi> for State {
    fn set_compliance(&mut self, compliance: ContractAddress) { self.compliance = compliance; }

    fn set_identity_registry(&mut self, identity_registry: ContractAddress) {
        self.identity_registry = identity_registry;
    }

    fn identity_registry(&self) -> ContractAddress { self.identity_registry }

    fn compliance(&self) -> ContractAddress { self.compliance }
}

// rewards state
impl IRewardTokenState<TokenId, TokenAmount, TokenIdVec, TokenAmountU64, StateApi> for TokenState {
    fn reward(&self) -> Option<RewardDeposited<TokenIdVec, TokenAmountU64>> { self.reward.clone() }

    fn attach_reward(&mut self, params: AddRewardParam<TokenIdVec, TokenAmountU64>) {
        self.metadata_url = params.metadata_url;
        self.reward = Some(params.reward);
    }

    fn new(metadata_url: MetadataUrl) -> Self { TokenState::new(metadata_url) }

    fn dec_locked_rewarded_amount(&mut self, amount: TokenAmountU64) {
        if let Some(reward) = self.reward.as_mut() {
            reward.token_amount.sub_assign(amount);
        }
    }
}
impl IRewardHolderState<TokenId, TokenAmount, StateApi> for HolderState {}
impl
    IRewardsState<
        TokenId,
        TokenAmount,
        TokenState,
        HolderState,
        TokenIdVec,
        TokenAmountU64,
        StateApi,
    > for State
{
    fn max_reward_token_id(&self) -> TokenId { self.max_reward_token_id }

    fn min_reward_token_id(&self) -> TokenId { self.min_reward_token_id }

    fn set_max_reward_token_id(&mut self, token_id: TokenId) {
        self.max_reward_token_id = token_id;
    }
}
