//! Rewards for security SFTs
use concordium_cis2::{
    AdditionalData, BurnEvent, Cis2Event, MintEvent, OnReceivingCis2DataParams, Receiver,
    TokenAmountU64, TokenIdVec, TokenMetadataEvent, Transfer,
};
use concordium_protocols::concordium_cis2_ext::{cis2_client, IsTokenAmount, PlusSubOne};
use concordium_protocols::rate::Rate;
use concordium_rwa_utils::conversions::to_additional_data;
use concordium_std::ops::{Sub, SubAssign};
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::*;
use crate::state::{RewardDeposited, RewardTokenState, TokenState};

#[derive(Debug, Serialize, Clone, SchemaType)]
pub struct TransferAddRewardParams {
    pub reward_token_contract: ContractAddress,
    pub reward_token_id:       TokenIdVec,
    pub data:                  AddRewardContractParam,
    pub token_id:              TokenId,
}

#[receive(
    contract = "security_sft_rewards",
    name = "transferAddReward",
    mutable,
    parameter = "TransferAddRewardParams",
    error = "Error"
)]
pub fn transfer_add_reward(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let sender = ctx.sender();
    let state = host.state();
    let is_authorized = state.address(&sender).is_some_and(
        |a: StateRef<crate::state::AddressState<ExternStateApi>>| {
            a.is_agent(&[AgentRole::Rewarder])
        },
    );
    ensure!(is_authorized, Error::Unauthorized);

    let params: TransferAddRewardParams = ctx.parameter_cursor().get()?;
    let curr_supply = state
        .token(&params.token_id)
        .ok_or(Error::InvalidTokenId)?
        .reward()
        .ok_or(Error::InvalidTokenId)?
        .supply;
    let (rewarded_amount, _) = params.data.rate.convert(&curr_supply.0)?;
    let rewarded_token_amount = TokenAmountU64(rewarded_amount);
    let receive_add_reward_receiver = Receiver::Contract(
        ctx.self_address(),
        OwnedEntrypointName::new_unchecked("receiveAddReward".to_string()),
    );

    // this contract should be an operator of the `sender` inside the token contract used for rewards
    cis2_client::transfer_single(host, &params.reward_token_contract, Transfer {
        token_id: params.reward_token_id,
        amount:   rewarded_token_amount,
        from:     sender,
        to:       receive_add_reward_receiver,
        data:     to_additional_data(params.data).map_err(|_| Error::InvalidRewardRate)?,
    })?;

    Ok(())
}

#[derive(Debug, Serialize, Clone, SchemaType)]
pub struct AddRewardContractParam {
    pub metadata_url: ContractMetadataUrl,
    pub rate:         Rate,
}

/// The parameters for the deposit function
/// The token id is the token id of the token to be deposited
/// The amount is the amount of tokens to be deposited hence any amount with size less than u64 can be deposited
pub type ReceiveAddRewardParam =
    OnReceivingCis2DataParams<TokenIdVec, TokenAmountU64, AddRewardContractParam>;

#[receive(
    contract = "security_sft_rewards",
    name = "receiveAddReward",
    mutable,
    parameter = "ReceiveAddRewardParam",
    error = "Error",
    enable_logger
)]
pub fn receive_add_reward(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let invoker = ctx.invoker();
    let token_contract = match ctx.sender() {
        Address::Account(_) => bail!(Error::Unauthorized),
        Address::Contract(sender) => sender,
    };
    let params: ReceiveAddRewardParam = ctx.parameter_cursor().get()?;

    let state = host.state_mut();
    let is_authorized = state.address(&Address::Account(invoker)).is_some_and(
        |a: StateRef<crate::state::AddressState<ExternStateApi>>| {
            a.is_agent(&[AgentRole::Rewarder])
        },
    );
    ensure!(is_authorized, Error::Unauthorized);
    let reward_metadata_url: MetadataUrl = params.data.metadata_url.into();
    let (min_reward_token_id, max_reward_token_id) = state.rewards_ids_range;

    let (next_token_id, next_token_metadata_url) = {
        let mut max_reward_token = state
            .token_mut(&max_reward_token_id)
            .ok_or(Error::InvalidTokenId)?;
        let max_reward_token = max_reward_token.reward_mut().ok_or(Error::InvalidTokenId)?;
        // next token id is the current max reward token id plus one
        let next_token_id = max_reward_token_id.plus_one();
        // Copy the existing metadata url to the new token
        // This metadata url will always be the default metadata url specified at the contract init
        let next_token_metadata_url = max_reward_token.metadata_url.clone();
        // Attach incoming reward to the current max reward token
        max_reward_token.attach_reward(reward_metadata_url.clone(), RewardDeposited {
            token_id: params.token_id,
            token_amount: params.amount,
            token_contract,
            rate: params.data.rate,
        });

        (next_token_id, next_token_metadata_url)
    };

    state.add_token(
        next_token_id,
        TokenState::Reward(RewardTokenState {
            metadata_url: next_token_metadata_url.clone(),
            reward:       None,
            supply:       TokenAmount::zero(),
        }),
    )?;
    state.rewards_ids_range = (min_reward_token_id, next_token_id);

    logger.log(&Event::Cis2(Cis2Event::TokenMetadata(TokenMetadataEvent {
        token_id:     max_reward_token_id,
        metadata_url: reward_metadata_url,
    })))?;
    logger.log(&Event::Cis2(Cis2Event::TokenMetadata(TokenMetadataEvent {
        token_id:     next_token_id,
        metadata_url: next_token_metadata_url,
    })))?;

    Ok(())
}

#[derive(Debug, Serialize, Clone, SchemaType)]
pub struct ClaimRewardsParams {
    pub owner:  Receiver,
    pub claims: Vec<ClaimRewardsParam>,
}

#[derive(Debug, Serialize, Clone, SchemaType)]
pub struct ClaimRewardsParam {
    pub token_id: TokenId,
    pub amount:   TokenAmount,
}

#[receive(
    contract = "security_sft_rewards",
    name = "claimRewards",
    mutable,
    parameter = "ClaimRewardsParams",
    error = "Error",
    enable_logger
)]
pub fn claim_rewards(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let params: ClaimRewardsParams = ctx.parameter_cursor().get()?;
    let owner_address = params.owner.address();
    ensure!(ctx.sender().eq(&owner_address), Error::Unauthorized);

    for claim in params.claims {
        let curr_reward_token_id = claim.token_id;
        let next_reward_token_id = claim.token_id.plus_one();

        // Calculated the amount of reward token to be burned and amount ot rewarded token to be transferred
        let (claim_amount, rewarded) = {
            let curr_reward_token = host
                .state()
                .token(&curr_reward_token_id)
                .ok_or(Error::InvalidTokenId)?;
            let curr_reward_token = curr_reward_token.reward().ok_or(Error::InvalidTokenId)?;
            match &curr_reward_token.reward {
                None => {
                    // Reward token has no attached rewards
                    // This can occur if the reward has been removed after attaching it to the token
                    (claim.amount, None)
                }
                Some(deposited_reward) => {
                    // Reward token has attached rewards
                    let (rewarded_amount, un_converted_amount) =
                        deposited_reward.rate.convert(&claim.amount.0)?;
                    let rewarded_amount = TokenAmountU64(rewarded_amount);
                    ensure!(
                        deposited_reward.token_amount.ge(&rewarded_amount),
                        Error::InsufficientDeposits
                    );
                    let claim_amount = claim.amount.sub(TokenAmountU64(un_converted_amount));
                    (
                        claim_amount,
                        Some((deposited_reward.clone(), rewarded_amount)),
                    )
                }
            }
        };

        let state = host.state_mut();
        {
            let mut curr_reward_token = state
                .token_mut(&curr_reward_token_id)
                .ok_or(Error::InvalidTokenId)?;
            let curr_reward_token = curr_reward_token
                .reward_mut()
                .ok_or(Error::InvalidTokenId)?;
            curr_reward_token.sub_assign_supply(claim.amount)?;

            if let Some((_, rewarded_amount)) = rewarded {
                curr_reward_token
                    .reward
                    .as_mut()
                    .ok_or(Error::InvalidTokenId)?
                    .token_amount
                    .sub_assign(rewarded_amount);
            }
        }

        {
            state
                .token_mut(&next_reward_token_id)
                .ok_or(Error::InvalidTokenId)?
                .reward_mut()
                .ok_or(Error::InvalidTokenId)?
                .add_assign_supply(claim.amount);
        }

        {
            let mut holder = state
                .address_mut(&owner_address)
                .ok_or(Error::InvalidAddress)?;
            let holder = holder.holder_mut().ok_or(Error::InvalidAddress)?;
            let holder = holder.active_mut().ok_or(Error::RecoveredAddress)?;
            holder.sub_assign_balance(&curr_reward_token_id, claim.amount)?;
            holder.add_assign_balance(&next_reward_token_id, claim.amount);
        }

        if let Some((deposits, rewarded_amount)) = rewarded {
            // Transfer the reward to the owner
            cis2_client::transfer_single(host, &deposits.token_contract, Transfer {
                from:     Address::Contract(ctx.self_address()),
                to:       params.owner.clone(),
                token_id: deposits.token_id,
                data:     AdditionalData::empty(),
                amount:   rewarded_amount,
            })?;
        }

        // burning the input token
        logger.log(&Event::Cis2(Cis2Event::Burn(BurnEvent {
            amount:   claim_amount,
            token_id: curr_reward_token_id,
            owner:    owner_address,
        })))?;
        // minting the reward token
        logger.log(&Event::Cis2(Cis2Event::Mint(MintEvent {
            amount:   claim_amount,
            token_id: next_reward_token_id,
            owner:    owner_address,
        })))?;
    }

    Ok(())
}
