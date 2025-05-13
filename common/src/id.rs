use std::{
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub struct Id<T> {
    raw: i64,
    marker: PhantomData<T>,
}

impl<T> Copy for Id<T> {}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Eq for Id<T> {}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.raw == other.raw
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.raw.cmp(&other.raw)
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.raw.hash(state);
    }
}

impl<T> Debug for Id<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Id")
            .field("raw", &self.raw)
            .finish_non_exhaustive()
    }
}

impl<T> Display for Id<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl<T> Serialize for Id<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Id<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        i64::deserialize(deserializer).map(Self::from)
    }
}

impl<T> From<i64> for Id<T> {
    fn from(raw: i64) -> Self {
        Id {
            raw,
            marker: PhantomData,
        }
    }
}

impl<T> Id<T> {
    pub fn raw(self) -> i64 {
        self.raw
    }
}

#[cfg(feature = "sqlx")]
mod sqlx {
    use std::error::Error;

    use sqlx::{
        Database, Decode, Encode, Type,
        encode::IsNull,
        postgres::{PgHasArrayType, PgTypeInfo},
    };

    use super::Id;

    impl<T, DB: Database> Type<DB> for Id<T>
    where
        i64: Type<DB>,
    {
        fn type_info() -> DB::TypeInfo {
            i64::type_info()
        }
    }

    impl<'q, T, DB: Database> Encode<'q, DB> for Id<T>
    where
        i64: Encode<'q, DB>,
    {
        fn encode_by_ref(
            &self,
            buf: &mut DB::ArgumentBuffer<'q>,
        ) -> Result<IsNull, Box<dyn Error + Send + Sync>> {
            self.raw.encode_by_ref(buf)
        }
    }

    impl<'r, T, DB: Database> Decode<'r, DB> for Id<T>
    where
        i64: Decode<'r, DB>,
    {
        fn decode(value: DB::ValueRef<'r>) -> Result<Self, Box<dyn Error + Send + Sync>> {
            i64::decode(value).map(Self::from)
        }
    }

    impl<T> PgHasArrayType for Id<T> {
        fn array_type_info() -> PgTypeInfo {
            i64::array_type_info()
        }
    }
}
