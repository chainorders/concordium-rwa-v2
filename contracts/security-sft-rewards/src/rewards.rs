//! Rewards for security SFTs

use concordium_cis2::{
    AdditionalData, BurnEvent, Cis2Event, MintEvent, OnReceivingCis2DataParams, Receiver,
    TokenAmountU64, TokenIdVec, TokenMetadataEvent, Transfer,
};
use concordium_protocols::concordium_cis2_ext::cis2_client;
use concordium_rwa_utils::conversions::exchange_rate::Rate;
use concordium_rwa_utils::conversions::to_additional_data;
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_rwa_utils::state_implementations::cis2_state::ICis2State;
use concordium_rwa_utils::state_implementations::rewards_state::{
    AddRewardParam, IRewardTokenState, IRewardsState, RewardDeposited,
};
use concordium_rwa_utils::state_implementations::tokens_state::ITokensState;
use concordium_std::ops::Sub;
use concordium_std::{
    bail, receive, Address, Get, HasCommonData, HasHost, HasLogger, HasReceiveContext, Host,
    Logger, ReceiveContext, *,
};

use super::error::*;
use super::state::State;
use super::types::*;

#[derive(Debug, Serialize, Clone, SchemaType)]
pub struct TransferAddRewardParams {
    pub token_contract: ContractAddress,
    pub token_id:       TokenIdVec,
    pub data:           AddRewardContractParam,
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
    ensure!(
        state.is_agent(&sender, vec![AgentRole::Rewarder]),
        Error::Unauthorized
    );
    let params: TransferAddRewardParams = ctx.parameter_cursor().get()?;
    let curr_supply: TokenAmount = state.supply_of(&state.tracked_token_id)?;
    let (rewarded_amount, _) = params.data.rate.convert(&curr_supply.0)?;
    let rewarded_token_amount = TokenAmountU64(rewarded_amount);
    let receive_add_reward_receiver = Receiver::Contract(
        ctx.self_address(),
        OwnedEntrypointName::new_unchecked("receiveAddReward".to_string()),
    );

    // this contract should be an operator of the `sender` inside the token contract used for rewards
    cis2_client::transfer_single(host, params.token_contract, Transfer {
        token_id: params.token_id,
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
    let token_contract = match ctx.sender() {
        Address::Account(_) => bail!(Error::Unauthorized),
        Address::Contract(sender) => sender,
    };
    let params: ReceiveAddRewardParam = ctx.parameter_cursor().get()?;

    let state = host.state_mut();
    // Check if the agent / instigator is authorized to add rewards
    ensure!(
        state.is_agent(&Address::Account(ctx.invoker()), vec![AgentRole::Rewarder]),
        Error::Unauthorized
    );
    let param_metadata_url: MetadataUrl = params.data.metadata_url.into();
    let (old_token_id, new_token_id) =
        state.add_reward(state.blank_reward_metadata_url.clone(), AddRewardParam {
            metadata_url: param_metadata_url.clone(),
            reward:       RewardDeposited {
                token_id: params.token_id,
                token_contract,
                token_amount: params.amount,
                rate: params.data.rate,
            },
        })?;

    logger.log(&Event::Cis2(Cis2Event::TokenMetadata(TokenMetadataEvent {
        token_id:     old_token_id,
        metadata_url: param_metadata_url,
    })))?;
    logger.log(&Event::Cis2(Cis2Event::TokenMetadata(TokenMetadataEvent {
        token_id:     new_token_id,
        metadata_url: state.blank_reward_metadata_url.clone(),
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
        let (state, state_builder) = host.state_and_builder();
        // exchange the reward token with the next reward token

        let reward_token = state.token(&claim.token_id)?.reward();

        // if the reward token has attached rewards.
        // Upon removing the rewards the token is left but the attached rewards are removed
        let (claim_amount, burned_token_id, minted_token_id) = if let Some(reward_token) =
            reward_token
        {
            let (rewarded_amount, un_converted_amount) =
                reward_token.rate.convert(&claim.amount.0)?;
            let rewarded_amount = TokenAmountU64(rewarded_amount);
            let claim_amount = claim.amount.sub(TokenAmountU64(un_converted_amount));
            let new_reward_token_id: TokenId = state.claim_rewards(
                &claim.token_id,
                &claim_amount,
                &owner_address,
                state_builder,
            )?;

            state.dec_locked_rewarded_amount(claim.token_id, rewarded_amount)?;
            cis2_client::transfer_single(host, reward_token.reward_token_contract(), Transfer {
                from:     Address::Contract(ctx.self_address()),
                to:       params.owner.clone(),
                token_id: reward_token.reward_token_id(),
                data:     AdditionalData::empty(),
                amount:   rewarded_amount,
            })?;

            (claim_amount, claim.token_id, new_reward_token_id)
        } else {
            let new_reward_token_id: TokenId = state.claim_rewards(
                &claim.token_id,
                &claim.amount,
                &owner_address,
                state_builder,
            )?;

            (claim.amount, claim.token_id, new_reward_token_id)
        };

        // burning the input token
        logger.log(&Event::Cis2(Cis2Event::Burn(BurnEvent {
            amount:   claim_amount,
            token_id: burned_token_id,
            owner:    owner_address,
        })))?;
        // minting the reward token
        logger.log(&Event::Cis2(Cis2Event::Mint(MintEvent {
            amount:   claim_amount,
            token_id: minted_token_id,
            owner:    owner_address,
        })))?;
    }

    Ok(())
}
