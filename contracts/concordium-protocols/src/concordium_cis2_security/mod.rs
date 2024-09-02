pub mod cis2_security_client;
pub mod compliance_client;
mod event;
pub mod identity_registry_client;
mod params_compliance;
mod params_identity_registry;
mod params_token;

use concordium_cis2::StandardIdentifier;
pub use event::*;
pub use params_compliance::*;
pub use params_identity_registry::*;
pub use params_token::*;
pub const COMPLIANCE_STANDARD_IDENTIFIER: StandardIdentifier =
    StandardIdentifier::new_unchecked("rwa_compliance");
pub const IDENTITY_REGISTRY_STANDARD_IDENTIFIER: StandardIdentifier =
    StandardIdentifier::new_unchecked("rwa_identity_registry");
