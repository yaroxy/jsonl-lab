use anyhow::Result;
use serde_json::Value;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonParser {
    Serde,
    Simd,
}

impl JsonParser {
    pub fn parse(&self, line: &[u8]) -> Result<Value> {
        match self {
            JsonParser::Serde => {
                let value = serde_json::from_slice(line)?;
                Ok(value)
            }
            JsonParser::Simd => {
                let mut bytes = line.to_vec();
                let value: Value = simd_json::serde::from_slice(&mut bytes)?;
                Ok(value)
            }
        }
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        JsonParser::Serde
    }
}

impl FromStr for JsonParser {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "serde" => Ok(JsonParser::Serde),
            "simd" => Ok(JsonParser::Simd),
            other => Err(format!(
                "unknown parser '{}'; expected 'serde' or 'simd'",
                other
            )),
        }
    }
}
