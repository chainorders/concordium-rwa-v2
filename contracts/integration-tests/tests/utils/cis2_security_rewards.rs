use concordium_smart_contract_testing::*;
use security_sft_rewards::rewards::{ClaimRewardsParams, TransferAddRewardParams};

use super::MAX_ENERGY;

pub fn transfer_add_reward(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &TransferAddRewardParams,
) -> ContractInvokeSuccess {
    chain
        .contract_update(
            Signer::with_one_key(),
            sender.address,
            sender.address.into(),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("transferAddReward"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("Transfer Add Reward")
}

pub fn claim_rewards(
    chain: &mut Chain,
    sender: &Account,
    contract: ContractAddress,
    contract_name: ContractName,
    payload: &ClaimRewardsParams,
) -> ContractInvokeSuccess {
    chain
        .contract_update(
            Signer::with_one_key(),
            sender.address,
            sender.address.into(),
            MAX_ENERGY,
            UpdateContractPayload {
                address:      contract,
                amount:       Amount::zero(),
                receive_name: OwnedReceiveName::construct_unchecked(
                    contract_name,
                    EntrypointName::new_unchecked("claimRewards"),
                ),
                message:      OwnedParameter::from_serial(payload).unwrap(),
            },
        )
        .expect("Transfer Add Reward")
}
