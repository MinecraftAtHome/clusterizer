use serde::{Deserialize, Serialize};
use sqlx::{Decode, Postgres, error::BoxDynError, postgres::PgValueRef};

use crate::errors::ValidateStateError;

#[derive(Clone, Hash, Debug, Serialize, Deserialize)]
pub enum ValidateState {
    Init,
    PendingValidation,
    Validated,
    NotNeeded,
    Inconclusive,
    Canceled,
}

impl Decode<'_, Postgres> for ValidateState {
    fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
        let int_bytes = value.as_bytes()?;
        let int_value = u32::from_ne_bytes(int_bytes.try_into()?);
        Ok(ValidateState::try_from(int_value)?)
    }
}

impl TryFrom<u32> for ValidateState {
    type Error = ValidateStateError;
    fn try_from(value: u32) -> Result<Self, ValidateStateError> {
        match value {
            0 => Ok(ValidateState::Init),
            1 => Ok(ValidateState::PendingValidation),
            2 => Ok(ValidateState::Validated),
            3 => Ok(ValidateState::NotNeeded),
            4 => Ok(ValidateState::Inconclusive),
            5 => Ok(ValidateState::Canceled),
            _ => Err(ValidateStateError::BadValidateState),
        }
    }
}
