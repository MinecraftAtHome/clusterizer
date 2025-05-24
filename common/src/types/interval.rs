use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize,
)]
pub struct Interval {
    pub months: i32,
    pub days: i32,
    pub microseconds: i64,
}

#[cfg(feature = "sqlx")]
mod sqlx {
    use sqlx::{
        Decode, Encode, Postgres, Type,
        encode::IsNull,
        error::BoxDynError,
        postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, types::PgInterval},
    };

    use super::Interval;

    impl From<Interval> for PgInterval {
        fn from(interval: Interval) -> Self {
            Self {
                months: interval.months,
                days: interval.days,
                microseconds: interval.microseconds,
            }
        }
    }

    impl From<PgInterval> for Interval {
        fn from(interval: PgInterval) -> Self {
            Self {
                months: interval.months,
                days: interval.days,
                microseconds: interval.microseconds,
            }
        }
    }

    impl Type<Postgres> for Interval {
        fn type_info() -> PgTypeInfo {
            PgInterval::type_info()
        }
    }

    impl Encode<'_, Postgres> for Interval {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            PgInterval::from(*self).encode_by_ref(buf)
        }
    }

    impl Decode<'_, Postgres> for Interval {
        fn decode(value: PgValueRef) -> Result<Self, BoxDynError> {
            PgInterval::decode(value).map(Self::from)
        }
    }

    impl PgHasArrayType for Interval {
        fn array_type_info() -> PgTypeInfo {
            PgInterval::array_type_info()
        }
    }
}
