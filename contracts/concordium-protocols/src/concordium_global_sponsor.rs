use concordium_std::{AccountAddress, Deserial, Serial, Serialize};

#[derive(Serialize)]
pub struct SponsoredParams<T> {
    pub signer: AccountAddress,
    pub params: T,
}

#[derive(Serialize)]
pub struct SponsoredParamsRaw {
    pub signer: AccountAddress,
    pub params: Vec<u8>,
}

impl SponsoredParamsRaw {
    pub fn bytes(&self) -> Option<Vec<u8>> {
        let mut bytes = Vec::new();
        let res = self.serial(&mut bytes);
        match res {
            Ok(()) => Some(bytes),
            Err(_) => None,
        }
    }
}

impl<T: Deserial> TryFrom<SponsoredParamsRaw> for SponsoredParams<T> {
    type Error = concordium_std::ParseError;

    fn try_from(value: SponsoredParamsRaw) -> Result<Self, Self::Error> {
        let params: T = {
            let mut cursor = concordium_std::Cursor::new(&value.params);
            Deserial::deserial(&mut cursor)?
        };

        Ok(SponsoredParams {
            signer: value.signer,
            params,
        })
    }
}
