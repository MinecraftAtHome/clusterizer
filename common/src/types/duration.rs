use serde::{Deserialize, Serialize};

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default, Serialize, Deserialize,
)]
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
    use sqlx::{
        Decode, Encode, Postgres, encode::IsNull, error::BoxDynError, postgres::PgArgumentBuffer,
        postgres::PgValueRef, postgres::types::PgInterval,
    };
    pub fn pg_interval_to_chrono_duration(pg_interval: PgInterval) -> chrono::Duration {
        let duration_days = (pg_interval.months * 30) + pg_interval.days;

        chrono::Duration::microseconds(pg_interval.microseconds)
            + chrono::Duration::days(duration_days as i64)
    }

    impl<'r> Decode<'r, Postgres> for Duration {
        fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
            let pg_interval = PgInterval::decode(value)?;
            Ok(pg_interval_to_chrono_duration(pg_interval).into())
        }
    }

    impl Encode<'_, Postgres> for Duration {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            let duration: chrono::Duration = self.0;
            let num_days = duration.num_days() as i32;
            let num_micros =
                (duration.num_seconds() % 86400) * 1_000_000 + duration.subsec_micros() as i64;
            let pg_interval = PgInterval {
                months: num_days / 30,
                days: num_days % 30,
                microseconds: num_micros,
            };
            pg_interval.encode_by_ref(buf)
        }
    }
}

#[cfg(all(test, feature = "sqlx"))]
mod tests {
    use super::sqlx::pg_interval_to_chrono_duration;
    use crate::types::duration;
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

    #[test]
    fn test_duration_conversion() {
        let chrono_duration = Duration::seconds(30);
        let wrapper_duration: duration::Duration = chrono_duration.into();
        let chrono_duration_converted: Duration = wrapper_duration.into();
        assert_eq!(chrono_duration, chrono_duration_converted);

        let chrono_minutes = Duration::minutes(5);
        let wrapper_duration: duration::Duration = chrono_minutes.into();
        let chrono_duration_converted: Duration = wrapper_duration.into();
        assert_eq!(chrono_minutes, chrono_duration_converted);

        let chrono_days = Duration::days(7);
        let wrapper_duration: duration::Duration = chrono_days.into();
        let chrono_duration_converted: Duration = wrapper_duration.into();
        assert_eq!(chrono_days, chrono_duration_converted);

        let chrono_micros = Duration::microseconds(1_500_000);
        let wrapper_duration: duration::Duration = chrono_micros.into();
        let chrono_duration_converted: Duration = wrapper_duration.into();
        assert_eq!(chrono_micros, chrono_duration_converted);
    }
}
