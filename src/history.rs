//! TWSE monthly trading history data API

use chrono::{Month, NaiveDate};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{Client, Error, Stock};

static ENDPOINT: &str = "https://www.twse.com.tw/exchangeReport/STOCK_DAY";
const FIELD_COUNT: usize = 9;

/// Trading summary of a single day
#[derive(Debug, Default, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DailyData {
    /// Be noted that `date` is `UTC+8`
    date: NaiveDate,
    volume: u64,
    transaction_price: f64,
    open_price: f64,
    high_price: f64,
    low_price: f64,
    close_price: f64,
    diff: f64,
    transaction: u64,
}

enum Column {
    Date,
    Volume,
    TransactionPrice,
    OpenPrice,
    HighPrice,
    LowPrice,
    ClosePrice,
    Diff,
    Transaction,
}

struct FieldMapper([Column; FIELD_COUNT]);

impl FieldMapper {
    fn new<'a>(fields: impl Iterator<Item = &'a str>) -> Result<Self, Error> {
        let map = fields
            .flat_map(|field| match field {
                "日期" => Ok(Column::Date),
                "成交股數" => Ok(Column::Volume),
                "成交金額" => Ok(Column::TransactionPrice),
                "開盤價" => Ok(Column::OpenPrice),
                "最高價" => Ok(Column::HighPrice),
                "最低價" => Ok(Column::LowPrice),
                "收盤價" => Ok(Column::ClosePrice),
                "漲跌價差" => Ok(Column::Diff),
                "成交筆數" => Ok(Column::Transaction),
                _ => Err(Error::IncompatibleApi),
            })
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| Error::IncompatibleApi)?;
        Ok(Self(map))
    }
    fn map(&self, data: &[Value; FIELD_COUNT]) -> Result<DailyData, Error> {
        let mut result = DailyData::default();
        macro_rules! parse {
            ($f:ident,$v:expr,$t:ty) => {
                paste::paste! {
                    result.$f = match $v{
                        Value::Number(x) => x.[<as_ $t>](),
                        Value::String(x) =>x.replace(",", "").parse().ok(),
                        _ => return Err(Error::IncompatibleApi),
                    }.ok_or(Error::IncompatibleApi)?
                }
            };
        }
        for (value, key) in data.iter().zip(self.0.iter()) {
            match key {
                Column::Date => {
                    result.date = value
                        .as_str()
                        .and_then(|x| {
                            x.split('/')
                                .map(|x| x.parse::<u32>().unwrap())
                                .collect_tuple()
                                .map(|(y, m, d)| {
                                    NaiveDate::from_ymd_opt(y as i32 + 1911, m, d).unwrap()
                                }) // Unwrap the Option<NaiveDate>
                        })
                        .ok_or(Error::IncompatibleApi)?
                }
                Column::Volume => parse!(volume, value, u64),
                Column::TransactionPrice => parse!(transaction_price, value, f64),
                Column::OpenPrice => parse!(open_price, value, f64),
                Column::HighPrice => parse!(high_price, value, f64),
                Column::LowPrice => parse!(low_price, value, f64),
                Column::ClosePrice => parse!(close_price, value, f64),
                Column::Diff => parse!(diff, value, f64),
                Column::Transaction => parse!(transaction, value, u64),
            }
        }
        Ok(result)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct RawMonthData {
    fields: [String; FIELD_COUNT],
    data: Vec<[Value; FIELD_COUNT]>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RawErrorMessage {
    stat: String,
}

/// newtype wrapper for the [`Client`] facilitating realtime data fetching
pub struct History<'a>(&'a Client);

impl Client {
    /// Get the history API client
    pub fn history(&self) -> History {
        History(self)
    }
}

impl History<'_> {
    /// Fetch the trading history of a stock in a specific month
    ///
    /// return every day that market open in the month
    pub async fn fetch(
        &self,
        month: Month,
        year: u16,
        stock: Stock,
    ) -> Result<Vec<DailyData>, Error> {
        let data = self.fetch_raw(month, year, stock).await?;
        let mapper = FieldMapper::new(data.fields.iter().map(|s| s.as_str()))?;
        data.data
            .iter()
            .map(|x| mapper.map(x))
            .collect::<Result<_, _>>()
    }
    async fn fetch_raw(
        &self,
        month: Month,
        year: u16,
        stock: Stock,
    ) -> Result<RawMonthData, Error> {
        let date = NaiveDate::from_ymd_opt(year as i32, month.number_from_month(), 1)
            .ok_or(Error::DateDoesNotExist)?
            .format("%Y%m%d")
            .to_string();

        let response = self
            .0
             .0
            .get(ENDPOINT)
            .query(&[
                ("response", "json"),
                ("date", &date),
                ("stockNo", &stock.code.to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::RateLimitExceeded);
        }

        let body = response.bytes().await?;
        match serde_json::from_slice(body.as_ref()) {
            Ok(x) => Ok(x),
            Err(_) => {
                let x: RawErrorMessage =
                    serde_json::from_slice(body.as_ref()).map_err(|_| Error::IncompatibleApi)?;
                Err(Error::StatMessage(x.stat))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    use super::*;
    use crate::{Stock, StockKind};

    #[tokio::test]
    async fn fetch() {
        let client = Client::new();
        let data = client
            .history()
            .fetch(
                Month::January,
                2021,
                Stock {
                    kind: StockKind::Live,
                    code: 2330,
                },
            )
            .await
            .unwrap();
        for item in &data {
            assert_eq!(item.date.year(), 2021)
        }
        assert_eq!(data.len(), 20);
    }
    #[tokio::test]
    async fn raw() {
        let client = Client::new();
        let data = client
            .history()
            .fetch_raw(
                Month::January,
                2021,
                Stock {
                    kind: StockKind::Live,
                    code: 2330,
                },
            )
            .await
            .unwrap();
        assert_eq!(data.data.len(), 20);
    }
}
