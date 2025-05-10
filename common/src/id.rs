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
