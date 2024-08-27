pub mod cis2;
pub mod cis2_conversions;
pub mod cis2_security;
pub mod cis2_security_rewards;
pub mod compliance;
pub mod euroe;
pub mod identity_registry;
pub mod market;
pub mod sft_security;

use concordium_smart_contract_testing::Energy;
const MAX_ENERGY: Energy = Energy { energy: 30000 };
