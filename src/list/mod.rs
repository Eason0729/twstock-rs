//! TWSE list api
//!
//! This module is technically not an API binding,
//! it's a http client with custom parser to list currently tradable stocks.

mod parser;

use chrono::NaiveDate;
use parser::RawContent;

use crate::{Client, Error, Stock, StockKind};

static ENDPOINT: &str = "https://isin.twse.com.tw/isin/C_public.jsp";

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// Industry category
pub enum Industry {
    /// consumer electronics
    Electronic,
    Cement,
    Food,
    Car,
    Motor,
    Steel,
    Semiconductor,
    Construction,
    Other(String),
}

impl Default for Industry {
    fn default() -> Self {
        Industry::Other("".to_string())
    }
}

impl From<&str> for Industry {
    fn from(value: &str) -> Self {
        match value {
            "電子零組件業" | "電子通路" | "電器電纜" => Industry::Electronic,
            "水泥工業" => Industry::Cement,
            "食品工業" => Industry::Food,
            "汽車工業" => Industry::Car,
            "電機機械" => Industry::Motor,
            "鋼鐵工業" => Industry::Steel,
            "半導體業" => Industry::Semiconductor,
            "建材營造業" => Industry::Construction,
            // FIXME: add more industries
            _ => Industry::Other(value.to_string()),
        }
    }
}

/// Stock information
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct StockInfo {
    /// Stock identifier
    pub id: Stock,
    /// chinese name abbreviation
    pub abbr: String,
    /// first trading date
    pub release_date: NaiveDate,
    /// industry category
    pub industry: Industry,
}

/// newtype wrapper for the [`Client`] facilitating list api
pub struct List<'a>(&'a Client);

impl Client {
    /// Get the list API client
    pub fn list(&self) -> List<'_> {
        List(self)
    }
}

impl List<'_> {
    /// Fetch the list of currently tradable stocks
    pub async fn fetch(&self, kind: StockKind) -> Result<Vec<StockInfo>, Error> {
        let raw = self.fetch_raw(kind).await?;
        let parser = RawContent(&raw);
        parser.parse()
    }
    async fn fetch_raw(&self, kind: StockKind) -> Result<Vec<u8>, Error> {
        let response = self
            .0
             .0
            .get(ENDPOINT)
            .query(&[("strMode", (kind as u8).to_string())])
            .send()
            .await?;
        if response.status().is_success() {
            Ok(response.bytes().await?.to_vec())
        } else {
            Err(Error::RateLimitExceeded)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::StockKind;

    #[tokio::test]
    #[ignore = "contain large amount of data"]
    async fn list() {
        let client = Client::new();
        let list = client.list();
        let data = list.fetch(StockKind::Live).await.unwrap();
        assert!(!data.is_empty());
    }
}
