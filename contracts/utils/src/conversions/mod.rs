use concordium_cis2::AdditionalData;
use concordium_std::Serial;

pub mod exchange_rate;

#[allow(clippy::result_unit_err)]
pub fn to_additional_data<S: Serial>(input: S) -> Result<AdditionalData, ()> {
    let mut bytes = Vec::new();
    input.serial(&mut bytes)?;
    Ok(bytes.into())
}
