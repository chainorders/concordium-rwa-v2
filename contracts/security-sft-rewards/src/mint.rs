use concordium_cis2::{AdditionalData, Cis2Event, MintEvent, OnReceivingCis2Params, Receiver};
use concordium_protocols::concordium_cis2_ext::IsTokenAmount;
use concordium_protocols::concordium_cis2_security::{
    compliance_client, identity_registry_client, CanTransferParam, MintedParam, TokenUId,
};
use concordium_rwa_utils::state_implementations::agent_with_roles_state::IAgentWithRolesState;
use concordium_rwa_utils::state_implementations::cis2_security_state::ICis2SecurityState;
use concordium_rwa_utils::state_implementations::rewards_state::IRewardsState;
use concordium_std::*;

use super::error::*;
use super::state::State;
use super::types::*;

#[receive(
    contract = "security_sft_rewards",
    name = "mint",
    enable_logger,
    mutable,
    parameter = "MintParam",
    error = "Error"
)]
pub fn mint(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let self_address = ctx.self_address();
    let params: MintParam = ctx.parameter_cursor().get()?;
    let can_mint = host.state().is_agent(&ctx.sender(), vec![AgentRole::Mint]);
    ensure!(can_mint, Error::Unauthorized);

    let state = host.state();
    let owner_address = params.owner.address();
    let compliance = state.compliance();
    let token_id = params.token_id;
    let mint_amount = params.amount;

    ensure!(mint_amount.gt(&TokenAmount::zero()), Error::InvalidAmount);
    // Ensure that the owner is verified
    ensure!(
        identity_registry_client::is_verified(host, state.identity_registry(), &owner_address)?,
        Error::UnVerifiedIdentity
    );

    let compliance_token = TokenUId::new(token_id, self_address);
    let compliance_can_transfer =
        compliance_client::can_transfer(host, compliance, &CanTransferParam {
            token_id: compliance_token,
            amount:   mint_amount,
            to:       owner_address,
        })?;
    ensure!(compliance_can_transfer, Error::InCompliantTransfer);

    // Minting Logic
    let (state, state_builder) = host.state_and_builder();
    ensure!(token_id.eq(&state.tracked_token_id), Error::InvalidTokenId);

    state.mint(&token_id, &mint_amount, &owner_address, state_builder)?;
    let reward_token_id = state.mint_rewards(&owner_address, &mint_amount, state_builder)?;
    // Notify compliance that the token has been minted
    compliance_client::minted(host, compliance, &MintedParam {
        token_id: compliance_token,
        amount:   mint_amount,
        owner:    owner_address,
    })?;
    // Log the mint event
    logger.log(&Event::Cis2(Cis2Event::Mint(MintEvent {
        token_id,
        amount: mint_amount,
        owner: owner_address,
    })))?;
    logger.log(&Event::Cis2(Cis2Event::Mint(MintEvent {
        token_id: reward_token_id,
        amount:   mint_amount,
        owner:    owner_address,
    })))?;
    // If the receiver is a contract: invoke the receive hook function.
    if let Receiver::Contract(address, function) = &params.owner {
        let parameter = OnReceivingCis2Params {
            token_id,
            amount: mint_amount,
            // From self because the minting is being done from deposited tokens in custody of the
            // current contract
            from: self_address.into(),
            data: AdditionalData::empty(),
        };
        host.invoke_contract(
            address,
            &parameter,
            function.as_entrypoint_name(),
            Amount::zero(),
        )?;
    }

    Ok(())
}
