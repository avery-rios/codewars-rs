use serde::{de, Deserialize, Serialize};
use std::{
    error,
    fmt::{self, Debug, Display},
    str::FromStr,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KataId([u8; 12]);
impl KataId {
    fn write_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in self.0 {
            write!(f, "{:02x}", i)?;
        }
        Ok(())
    }
}

impl Debug for KataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_fmt(f)
    }
}
impl Display for KataId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write_fmt(f)
    }
}

#[derive(Debug)]
pub enum InvalidId {
    InvalidLen(usize),
    InvalidWord(usize, std::num::ParseIntError),
}

impl Display for InvalidId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLen(l) => write!(f, "Invalid length {}, expected 24", l),
            Self::InvalidWord(pos, e) => write!(f, "Parse int error at {}: {}", pos, e),
        }
    }
}
impl error::Error for InvalidId {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::InvalidLen(_) => None,
            Self::InvalidWord(_, w) => Some(w),
        }
    }
}

impl FromStr for KataId {
    type Err = InvalidId;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 24 {
            return Err(InvalidId::InvalidLen(s.len()));
        }
        let mut ret = [0; 12];
        for i in 0..12 {
            ret[i] = u8::from_str_radix(&s[(i * 2)..(i * 2 + 2)], 16)
                .map_err(|e| InvalidId::InvalidWord(i, e))?;
        }
        Ok(KataId(ret))
    }
}
impl TryFrom<&str> for KataId {
    type Error = InvalidId;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Serialize for KataId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
impl<'de> Deserialize<'de> for KataId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> de::Visitor<'de> for Visitor {
            type Value = KataId;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Kata id")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                KataId::from_str(v).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

#[cfg(test)]
mod test {
    use super::KataId;
    use hex_literal::hex;
    use std::str::FromStr;

    #[test]
    fn as_str() {
        assert_eq!(
            KataId(hex!("5277c8a221e209d3f6000b56")).to_string(),
            "5277c8a221e209d3f6000b56"
        );
    }

    #[test]
    fn from_str() {
        assert_eq!(
            KataId::from_str("5277c8a221e209d3f6000b56").unwrap(),
            KataId(hex!("5277c8a221e209d3f6000b56"))
        );
    }
}
