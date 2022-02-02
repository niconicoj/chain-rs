use std::fmt::Display;

#[cfg(feature = "serde")]
use serde::{
    de::{self, Visitor},
    Deserialize, Serialize,
};

#[cfg(feature = "serde")]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Hash(u128, u128);

#[cfg(not(feature = "serde"))]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub struct Hash(u128, u128);

pub trait Hashable {
    fn bytes(&self) -> Vec<u8>;

    fn make_hash(&self) -> Hash {
        Hash::from_bytes(&self.bytes())
    }
}

impl Hash {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, bytes);

        Self(
            u128::from_le_bytes(to_byte_array(hash[0..16].to_vec())),
            u128::from_le_bytes(to_byte_array(hash[16..32].to_vec())),
        )
    }

    pub fn bytes(&self) -> Vec<u8> {
        [self.0.to_le_bytes(), self.1.to_le_bytes()].concat()
    }
}

fn to_byte_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
    v.try_into()
        .unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
}

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            hex::encode(self.0.to_le_bytes()),
            hex::encode(self.1.to_le_bytes())
        )?;
        Ok(())
    }
}

#[cfg(feature = "serde")]
impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bytes: Vec<u8> = self.0.to_le_bytes().to_vec();
        bytes.extend(self.1.to_le_bytes());
        serializer.serialize_str(&hex::encode(bytes))
    }
}

struct HashVisitor;

#[cfg(feature = "serde")]
impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid hexadecimal hash")
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut lower = hex::decode(&v[0..16]).unwrap();
        let upper = hex::decode(&v[16..32]).unwrap();
        lower.extend_from_slice(&upper);

        Ok(Hash::from_bytes(&lower))
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(HashVisitor)
    }
}
