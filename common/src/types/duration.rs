use ::sqlx::error::BoxDynError;
use chrono::Duration;
use serde::{Deserialize, Serialize};

// modified implementation of https://github.com/launchbadge/sqlx/blob/main/sqlx-postgres/src/types/interval.rs
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Default, Serialize, Deserialize)]
pub struct ClDuration {
    pub months: i32,
    pub days: i32,
    pub microseconds: i64,
}

impl TryFrom<Duration> for ClDuration {
    type Error = BoxDynError;
    fn try_from(value: Duration) -> Result<Self, BoxDynError> {
        value
            .num_nanoseconds()
            .map_or::<Result<_, Self::Error>, _>(
                Err("Overflow has occurred for PostgreSQL `INTERVAL`".into()),
                |nanoseconds| {
                    if nanoseconds % 1000 != 0 {
                        return Err(
                            "PostgreSQL `INTERVAL` does not support nanoseconds precision".into(),
                        );
                    }
                    Ok(())
                },
            )?;

        value.num_microseconds().map_or(
            Err("Overflow has occurred for PostgreSQL `INTERVAL`".into()),
            |microseconds| {
                Ok(Self {
                    months: 0,
                    days: 0,
                    microseconds,
                })
            },
        )
    }
}

#[cfg(feature = "sqlx")]
mod sqlx {
    use std::mem;

    use crate::types::duration::ClDuration;
    use byteorder::{NetworkEndian, ReadBytesExt};
    use sqlx::encode::IsNull;
    use sqlx::error::BoxDynError;
    use sqlx::postgres::{PgArgumentBuffer, PgValueFormat, PgValueRef};
    use sqlx::{Decode, Encode, Postgres};

    impl<'de> Decode<'de, Postgres> for ClDuration {
        fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
            match value.format() {
                PgValueFormat::Binary => {
                    let mut buf = value.as_bytes()?;
                    let microseconds = buf.read_i64::<NetworkEndian>()?;
                    let days = buf.read_i32::<NetworkEndian>()?;
                    let months = buf.read_i32::<NetworkEndian>()?;

                    Ok(ClDuration {
                        months,
                        days,
                        microseconds,
                    })
                }

                // TODO: Implement parsing of text mode
                PgValueFormat::Text => Err(
                    "not implemented: decode `INTERVAL` in text mode (unprepared queries)".into(),
                ),
            }
        }
    }

    impl Encode<'_, Postgres> for ClDuration {
        fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
            buf.extend(&self.microseconds.to_be_bytes());
            buf.extend(&self.days.to_be_bytes());
            buf.extend(&self.months.to_be_bytes());

            Ok(IsNull::No)
        }

        fn size_hint(&self) -> usize {
            2 * mem::size_of::<i64>()
        }
    }
}
