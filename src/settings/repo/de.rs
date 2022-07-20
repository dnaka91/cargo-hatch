//! Custom deserializers for [`serde`].

use std::{
    fmt::{self, Display},
    marker::PhantomData,
    str::FromStr,
};

use serde::de::{Deserializer, Visitor};

/// Deserialize any type from its text form, that implements [`FromStr`].
pub fn from_str<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    deserializer.deserialize_str(FromStrVisitor { ty: PhantomData })
}

struct FromStrVisitor<T> {
    ty: PhantomData<T>,
}

impl<'de, T> Visitor<'de> for FromStrVisitor<T>
where
    T: FromStr,
    T::Err: Display,
{
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "type {} in its text form", std::any::type_name::<T>())
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse().map_err(E::custom)
    }
}
