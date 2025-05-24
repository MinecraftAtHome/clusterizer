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
    use crate::types::interval::Interval;
    use sqlx::{
        Decode, Encode, Postgres, encode::IsNull, error::BoxDynError, postgres::PgArgumentBuffer,
        postgres::PgValueRef, postgres::types::PgInterval,
    };

    impl From<Interval> for PgInterval {
        fn from(interval: Interval) -> PgInterval {
            Self {
                months: interval.months,
                days: interval.days,
                microseconds: interval.microseconds,
            }
        }
    }

    impl From<PgInterval> for Interval {
        fn from(interval: PgInterval) -> Interval {
            Self {
                months: interval.months,
                days: interval.days,
                microseconds: interval.microseconds,
            }
        }
    }

    impl<'r> Decode<'r, Postgres> for Interval {
        fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
            let pg_interval = PgInterval::decode(value)?;
            Ok(Self {
                months: pg_interval.months,
                days: pg_interval.days,
                microseconds: pg_interval.microseconds,
            })
        }
    }

    impl Encode<'_, Postgres> for Interval {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            let pg_interval = PgInterval {
                months: self.months,
                days: self.days,
                microseconds: self.microseconds,
            };
            pg_interval.encode_by_ref(buf)
        }
    }
}
