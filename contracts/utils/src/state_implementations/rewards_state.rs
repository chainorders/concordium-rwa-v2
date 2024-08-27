use concordium_protocols::concordium_cis2_ext::{IsTokenAmount, IsTokenId, PlusSubOne};
use concordium_std::{
    ensure, Address, ContractAddress, HasStateApi, MetadataUrl, Serialize, StateBuilder,
};

use super::cis2_state::{Cis2StateError, ICis2State, ICis2TokenState};
use super::holders_state::{HolderStateError, IHolderState};
use crate::conversions::exchange_rate::Rate;

pub enum RewardsStateError {
    InsufficientFunds,
    InvalidTokenId,
    InvalidAmount,
}

impl From<Cis2StateError> for RewardsStateError {
    fn from(e: Cis2StateError) -> Self {
        match e {
            Cis2StateError::InvalidTokenId => RewardsStateError::InvalidTokenId,
            Cis2StateError::InsufficientFunds => RewardsStateError::InsufficientFunds,
            Cis2StateError::InvalidAmount => RewardsStateError::InvalidAmount,
        }
    }
}

impl From<HolderStateError> for RewardsStateError {
    fn from(e: HolderStateError) -> Self {
        match e {
            HolderStateError::AmountTooLarge => RewardsStateError::InsufficientFunds,
        }
    }
}

pub type RewardsStateResult<T> = Result<T, RewardsStateError>;

#[derive(Serialize, Clone)]
pub struct RewardDeposited<TDeposit: IsTokenId, ADeposit: IsTokenAmount> {
    pub token_id:       TDeposit,
    pub token_contract: ContractAddress,
    pub token_amount:   ADeposit,
    pub rate:           Rate,
}

#[derive(Serialize, Clone)]
pub struct RewardContract<T: IsTokenId, A: IsTokenAmount> {
    pub token_id:     T,
    pub token_amount: A,
}

impl<TDeposit: IsTokenId, ADeposit: IsTokenAmount> RewardDeposited<TDeposit, ADeposit> {
    #[inline]
    pub fn reward_token_id(&self) -> TDeposit { self.token_id.clone() }

    #[inline]
    pub fn reward_token_contract(&self) -> ContractAddress { self.token_contract }
}

pub trait IRewardTokenState<T, A, TDeposit: IsTokenId, ADeposit: IsTokenAmount, S>:
    ICis2TokenState<A> {
    fn new(metadata_url: MetadataUrl) -> Self;
    fn reward(&self) -> Option<RewardDeposited<TDeposit, ADeposit>>;
    fn attach_reward(&mut self, params: AddRewardParam<TDeposit, ADeposit>);
    fn dec_locked_rewarded_amount(&mut self, amount: ADeposit);
}

pub trait IRewardHolderState<T, A, S: HasStateApi>: IHolderState<T, A, S> {}

pub struct AddRewardParam<TDeposit: IsTokenId, ADeposit: IsTokenAmount> {
    pub metadata_url: MetadataUrl,
    pub reward:       RewardDeposited<TDeposit, ADeposit>,
}

pub trait IRewardsState<
    T: IsTokenId+PlusSubOne<T>+Clone,
    A: IsTokenAmount,
    TTokenState: IRewardTokenState<T, A, TDeposit, ADeposit, S>,
    THolderState: IRewardHolderState<T, A, S>,
    TDeposit: IsTokenId,
    ADeposit: IsTokenAmount,
    S: HasStateApi,
>: ICis2State<T, A, TTokenState, THolderState, S> {
    fn max_reward_token_id(&self) -> T;
    fn min_reward_token_id(&self) -> T;
    fn set_max_reward_token_id(&mut self, token_id: T);

    fn add_reward(
        &mut self,
        default_reward_metadata_url: MetadataUrl,
        token_state: AddRewardParam<TDeposit, ADeposit>,
    ) -> RewardsStateResult<(T, T)> {
        let max_reward_token_id = self.max_reward_token_id();
        let new_reward_token_id = max_reward_token_id.plus_one();
        let _ = self.add_token(
            new_reward_token_id.clone(),
            TTokenState::new(default_reward_metadata_url),
        );
        self.set_max_reward_token_id(new_reward_token_id.clone());

        // attach reward to the max reward token id
        self.attach_reward(max_reward_token_id.clone(), token_state)?;
        // Results are ignored because a new token id is being generated
        Ok((max_reward_token_id, new_reward_token_id))
    }

    fn attach_reward(
        &mut self,
        token_id: T,
        token_state: AddRewardParam<TDeposit, ADeposit>,
    ) -> RewardsStateResult<()> {
        ensure!(
            token_id.lt(&self.max_reward_token_id()) && token_id.ge(&self.min_reward_token_id()),
            RewardsStateError::InvalidTokenId
        );

        self.tokens_mut()
            .entry(token_id)
            .occupied_or(RewardsStateError::InvalidTokenId)?
            .modify(|e| e.attach_reward(token_state));
        Ok(())
    }

    fn claim_rewards(
        &mut self,
        token_id: &T,
        amount: &A,
        owner: &Address,
        state_builder: &mut StateBuilder<S>,
    ) -> RewardsStateResult<T> {
        ensure!(
            token_id.lt(&self.max_reward_token_id()) && token_id.ge(&self.min_reward_token_id()),
            RewardsStateError::InvalidTokenId
        );

        let new_reward_token_id = token_id.plus_one();
        self.burn(token_id, amount, owner)?;
        self.mint(&new_reward_token_id, amount, owner, state_builder)?;

        Ok(new_reward_token_id)
    }

    fn dec_locked_rewarded_amount(
        &mut self,
        token_id: T,
        amount: ADeposit,
    ) -> RewardsStateResult<()> {
        ensure!(
            token_id.lt(&self.max_reward_token_id()) && token_id.ge(&self.min_reward_token_id()),
            RewardsStateError::InvalidTokenId
        );

        self.tokens_mut()
            .entry(token_id)
            .occupied_or(RewardsStateError::InvalidTokenId)?
            .modify(|e| e.dec_locked_rewarded_amount(amount));
        Ok(())
    }

    fn transfer_rewards(
        &mut self,
        from: &Address,
        to: &Address,
        amount: &A,
        state_builder: &mut StateBuilder<S>,
    ) -> RewardsStateResult<Vec<RewardContract<T, A>>> {
        let max_reward_token_id: T = self.max_reward_token_id();
        let min_reward_token_id: T = self.min_reward_token_id();

        let mut curr_reward_token_id = max_reward_token_id;
        let mut remaining_balance = *amount;
        let mut res = Vec::new();

        while curr_reward_token_id.ge(&min_reward_token_id) {
            let curr_token_balance = self.balance_of(from, &curr_reward_token_id);
            let transfer_balance = curr_token_balance.min(remaining_balance);

            self.transfer(
                from,
                to,
                &curr_reward_token_id,
                &transfer_balance,
                state_builder,
            )?;
            res.push(RewardContract {
                token_id:     curr_reward_token_id.clone(),
                token_amount: transfer_balance,
            });
            curr_reward_token_id = curr_reward_token_id.sub_one();
            remaining_balance = remaining_balance.sub(transfer_balance);
            if remaining_balance.is_zero() {
                break;
            }
        }

        if !remaining_balance.is_zero() {
            return Err(RewardsStateError::InvalidAmount);
        }

        Ok(res)
    }

    /// Mints rewards for the given address and amount.
    /// It would be good to ensure that the final balance is not too large.
    ///
    /// # Arguments
    /// * `to` - The address to mint the rewards to.
    /// * `amount` - The amount of rewards to mint.
    /// * `state_builder` - The state builder to use for the minting operation.
    ///
    /// # Returns
    /// A vector of `Reward` objects representing the minted rewards.
    ///
    /// # Errors
    /// Returns a `RewardsStateError` if there is an issue minting the rewards.
    fn mint_rewards(
        &mut self,
        to: &Address,
        amount: &A,
        state_builder: &mut StateBuilder<S>,
    ) -> RewardsStateResult<T> {
        let max_reward_token_id: T = self.max_reward_token_id();
        self.mint(&max_reward_token_id, amount, to, state_builder)?;
        Ok(max_reward_token_id)
    }
}
