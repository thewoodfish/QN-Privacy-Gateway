//! Privacy mode behavior and cache policy.

use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrivacyMode {
    Strict,
    Balanced,
    Dev,
}

impl PrivacyMode {
    pub fn should_cache(self, method: &str) -> bool {
        match self {
            PrivacyMode::Strict => is_cacheable_method(method),
            PrivacyMode::Balanced => {
                matches!(method, "getLatestBlockhash" | "getSlot" | "getBalance")
            }
            PrivacyMode::Dev => false,
        }
    }

    pub fn should_normalize_outbound(self) -> bool {
        !matches!(self, PrivacyMode::Dev)
    }
}

impl FromStr for PrivacyMode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "strict" => Ok(PrivacyMode::Strict),
            "balanced" => Ok(PrivacyMode::Balanced),
            "dev" => Ok(PrivacyMode::Dev),
            _ => Err(()),
        }
    }
}

pub fn is_cacheable_method(method: &str) -> bool {
    matches!(
        method,
        "getAccountInfo" | "getBalance" | "getLatestBlockhash" | "getSlot" | "getBlock"
    )
}

impl fmt::Display for PrivacyMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            PrivacyMode::Strict => "strict",
            PrivacyMode::Balanced => "balanced",
            PrivacyMode::Dev => "dev",
        };
        write!(f, "{}", value)
    }
}
