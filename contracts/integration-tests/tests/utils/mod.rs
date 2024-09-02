pub mod cis2;
pub mod cis2_conversions;
pub mod cis2_security;
pub mod cis2_security_rewards;
pub mod compliance;
pub mod euroe;
pub mod identity_registry;
pub mod security_mint_fund_client;
pub mod security_p2p_trading_client;
pub mod security_sft_rewards_client;
pub mod security_sft_single_client;

use concordium_smart_contract_testing::Energy;
const MAX_ENERGY: Energy = Energy { energy: 30000 };
