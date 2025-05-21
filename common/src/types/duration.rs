use serde::{Deserialize, Serialize};

// modified implementation of https://github.com/launchbadge/sqlx/blob/main/sqlx-postgres/src/types/interval.rs
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Default, Serialize, Deserialize)]
pub struct Duration(chrono::Duration);

impl From<Duration> for chrono::Duration {
    fn from(duration: Duration) -> Self {
        duration.0
    }
}

impl From<chrono::Duration> for Duration {
    fn from(duration: chrono::Duration) -> Self {
        Duration(duration)
    }
}

#[cfg(feature = "sqlx")]
mod sqlx {
    use crate::types::duration::Duration;
    use sqlx::encode::IsNull;
    use sqlx::error::BoxDynError;
    use sqlx::postgres::types::PgInterval;
    use sqlx::postgres::{PgArgumentBuffer, PgValueRef};
    use sqlx::{Decode, Encode, Postgres};

    pub fn pg_interval_to_chrono_duration(pg_interval: PgInterval) -> chrono::Duration {
        let duration_days = (pg_interval.months * 30) + pg_interval.days;

        chrono::Duration::microseconds(pg_interval.microseconds)
            + chrono::Duration::days(duration_days as i64)
    }

    impl<'de> Decode<'de, Postgres> for Duration {
        fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
            let pg_interval = PgInterval::decode(value)?;
            Ok(pg_interval_to_chrono_duration(pg_interval).into())
        }
    }

    impl Encode<'_, Postgres> for Duration {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            let duration: chrono::Duration = self.0;
            let pg_interval = PgInterval::try_from(duration)?;
            pg_interval.encode_by_ref(buf)
        }
    }
}

#[cfg(all(test, feature = "sqlx"))]
mod tests {
    use super::sqlx::pg_interval_to_chrono_duration;
    use chrono::Duration;
    use sqlx::postgres::types::PgInterval;

    #[test]
    fn test_pg_interval_to_chrono_duration() {
        let interval = PgInterval {
            months: 0,
            days: 0,
            microseconds: 5_000_000,
        };
        let duration = pg_interval_to_chrono_duration(interval);
        assert_eq!(duration, Duration::seconds(5));

        let interval = PgInterval {
            months: 0,
            days: 2,
            microseconds: 0,
        };
        let duration = pg_interval_to_chrono_duration(interval);
        assert_eq!(duration, Duration::days(2));

        let interval = PgInterval {
            months: 1,
            days: 0,
            microseconds: 0,
        };
        let duration = pg_interval_to_chrono_duration(interval);
        assert_eq!(duration, Duration::days(30));
    }
}
