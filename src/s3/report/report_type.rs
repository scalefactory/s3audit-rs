use anyhow::Result;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum ReportType {
    Csv,
    Text,
}

impl Default for ReportType {
    fn default() -> Self {
        Self::Text
    }
}

impl FromStr for ReportType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            "csv"  => Ok(Self::Csv),
            "text" => Ok(Self::Text),
            _      => Err(anyhow::anyhow!("Unknown Report Type")),
        }
    }
}
