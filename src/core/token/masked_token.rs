use serde::{Deserialize, Deserializer};
use std::fmt;

pub struct MaskedToken {
    token: String,
}

impl MaskedToken {
    fn new(token: String) -> Self {
        Self { token }
    }
}

impl fmt::Display for MaskedToken {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (first_part, rest) = self.token.split_at(5);
        let masked_rest = "*".repeat(rest.len());
        write!(f, "{}{}", first_part, masked_rest)
    }
}

impl fmt::Debug for MaskedToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MaskedToken({})", self)
    }
}

impl From<String> for MaskedToken {
    fn from(token: String) -> Self {
        MaskedToken::new(token)
    }
}

impl From<&str> for MaskedToken {
    fn from(token: &str) -> Self {
        MaskedToken::new(token.to_string())
    }
}

impl AsRef<str> for MaskedToken {
    fn as_ref(&self) -> &str {
        &self.token
    }
}

impl<'de> Deserialize<'de> for MaskedToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let token = String::deserialize(deserializer)?;
        Ok(MaskedToken::new(token))
    }
}
