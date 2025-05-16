use serde::{Deserialize, Serialize};
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
#[cfg(feature = "sqlx")]
mod sqlx {
    use sqlx::{error::BoxDynError, postgres::PgValueRef, Decode, Postgres};

    use super::ValidateState;

    impl Decode<'_, Postgres> for ValidateState {
        fn decode(value: PgValueRef<'_>) -> Result<Self, BoxDynError> {
            Ok(ValidateState::try_from(u32::from_ne_bytes(value.as_bytes()?.try_into()?))?)
        }
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
