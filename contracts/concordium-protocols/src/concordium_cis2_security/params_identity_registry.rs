use concordium_std::*;

pub type AttributeTag = u8;
pub type AttributeValue = String;
pub type Issuer = ContractAddress;

#[derive(Serialize, SchemaType)]
pub struct IdentityAttribute {
    pub tag:   AttributeTag,
    pub value: AttributeValue,
}

#[derive(Serialize, SchemaType)]
pub struct IdentityCredential {
    pub issuer: Issuer,
    pub key:    PublicKeyEd25519,
}

#[derive(Serialize, SchemaType)]
pub struct Identity {
    pub attributes:  Vec<IdentityAttribute>,
    pub credentials: Vec<IdentityCredential>,
}
