use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_rwa_utils::state_implementations::token_deposits_state::{
    DepositedStateError, DepositedTokenState, IDepositedTokensState,
};
use concordium_std::*;

use super::types::{ExchangeRate, *};

#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "S")]
/// Represents the state of the market contract.
pub struct State<S=StateApi> {
    /// Tokens which can be received by the market contract and kept in custody
    /// of the contract / deposited with the contract.
    deposited_tokens:     StateMap<TokenOwnerUId, DepositedTokenState<Cis2TokenAmount>, S>,
    /// Tokens which are listed for sale.
    listed_tokens:        StateMap<TokenOwnerUId, Vec<ExchangeRate>, S>,
    pub commission:       Rate,
    /// Tokens which can be used to pay for the commission & listed tokens.
    payment_tokens:       StateSet<TokenUId, S>,
    /// Contracts whose tokens can be exchanged for payment tokens.
    sell_token_contracts: StateSet<ContractAddress, S>,
}

impl State<StateApi> {
    pub fn new(
        commission: Rate,
        payment_tokens: Vec<TokenUId>,
        sell_token_contracts: Vec<ContractAddress>,
        state_builder: &mut StateBuilder,
    ) -> Self {
        let mut state = Self {
            deposited_tokens: state_builder.new_map(),
            listed_tokens: state_builder.new_map(),
            commission,
            payment_tokens: state_builder.new_set(),
            sell_token_contracts: state_builder.new_set(),
        };

        for token in payment_tokens {
            state.payment_tokens.insert(token);
        }

        for contract in sell_token_contracts {
            state.sell_token_contracts.insert(contract);
        }

        state
    }

    pub fn sell_token_contracts(&self) -> Vec<ContractAddress> {
        self.sell_token_contracts
            .iter()
            .map(|r| r.to_owned())
            .collect()
    }

    pub fn add_sell_token_contract(&mut self, contract: ContractAddress) {
        self.sell_token_contracts.insert(contract);
    }

    pub fn payment_tokens(&self) -> Vec<TokenUId> {
        self.payment_tokens.iter().map(|r| r.to_owned()).collect()
    }

    pub fn add_payment_token(&mut self, token: TokenUId) { self.payment_tokens.insert(token); }

    pub fn can_be_paid_by(&self, token_uid: &TokenUId) -> bool {
        self.payment_tokens.contains(token_uid)
    }

    pub fn can_list(&self, token_uid: &TokenUId) -> bool {
        self.sell_token_contracts.contains(&token_uid.contract)
    }

    pub fn unlisted_amount(&self, id: &TokenOwnerUId) -> Cis2TokenAmount {
        self.balance_of_unlocked(id)
    }

    pub fn add_or_replace_listed(
        &mut self,
        token_id: TokenOwnerUId,
        supply: Cis2TokenAmount,
        exchange_rates: Vec<ExchangeRate>,
    ) -> Result<(), DepositedStateError> {
        self.set_locked_deposits(&token_id, supply)?;
        let _ = self.listed_tokens.insert(token_id, exchange_rates);

        Ok(())
    }

    pub fn remove_listed(&mut self, id: &TokenOwnerUId) -> Result<(), DepositedStateError> {
        self.set_locked_deposits(id, Cis2TokenAmount::zero())?;
        self.listed_tokens.remove(id);
        Ok(())
    }

    pub fn listed_amount(&self, id: &TokenOwnerUId) -> Cis2TokenAmount {
        self.balance_of_locked(id)
    }

    pub fn consume_listed(
        &mut self,
        id: &TokenOwnerUId,
        amount: Cis2TokenAmount,
    ) -> Result<bool, DepositedStateError> {
        let is_removed = self
            .burn_locked_deposits(id, amount)?
            .eq(&Cis2TokenAmount::zero())
            .then(|| self.listed_tokens.remove(id))
            .is_some();
        Ok(is_removed)
    }

    pub fn is_listed(&self, id: &TokenOwnerUId) -> bool { self.listed_tokens.get(id).is_some() }

    pub fn get_listed(&self, id: &TokenOwnerUId) -> Option<(Cis2TokenAmount, Vec<ExchangeRate>)> {
        let balance = self.balance_of_locked(id);
        let rates = self.listed_tokens.get(id);

        rates.map(|rates| (balance, rates.to_owned()))
    }
}

impl IDepositedTokensState<TokenOwnerUId, Cis2TokenAmount, StateApi> for State {
    fn tokens(&self) -> &StateMap<TokenOwnerUId, DepositedTokenState<Cis2TokenAmount>, StateApi> {
        &self.deposited_tokens
    }

    fn tokens_mut(
        &mut self,
    ) -> &mut StateMap<TokenOwnerUId, DepositedTokenState<Cis2TokenAmount>, StateApi> {
        &mut self.deposited_tokens
    }
}
