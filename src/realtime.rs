//! TWSE realtime data API

use super::*;
use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::Value;

static ENDPOINT: &str = "https://mis.twse.com.tw/stock/api/getStockInfo.jsp";

fn default_json_number() -> Value {
    Value::String("1".to_owned())
}

/// realtime frame data from TWSE
#[derive(Debug, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RealTimeData {
    pub price: f64,
    pub volume: u64,
    pub history_volume: u64,
    pub update_at: DateTime<FixedOffset>,
    /// Be noted that `recent_trading_date` is `UTC+8`
    pub recent_trading_date: NaiveDate,
    pub name: String,
    pub opening_price: f64,
    pub histroy_high_price: f64,
    pub histroy_low_price: f64,
    pub yesterday_closing_price: f64,
    pub limit_up_price: f64,
    pub limit_down_price: f64,
}

/// Raw frame data from TWSE
#[derive(Debug, Serialize, Deserialize)]
struct FrameData {
    #[serde(rename = "z")]
    price: Value,
    #[serde(rename = "tv")]
    volume: Value,
    #[serde(rename = "v", default = "default_json_number")]
    history_volume: Value,
    #[serde(rename = "tlong")]
    update_at: Value,
    #[serde(rename = "d")]
    recent_trading_date: Value,
    #[serde(rename = "n")]
    name: String,
    #[serde(rename = "o")]
    opening_price: Value,
    #[serde(rename = "h")]
    histroy_high_price: Value,
    #[serde(rename = "l")]
    histroy_low_price: Value,
    #[serde(rename = "y")]
    yesterday_closing_price: Value,
    #[serde(rename = "u")]
    limit_up_price: Value,
    #[serde(rename = "w")]
    limit_down_price: Value,
}

impl TryFrom<FrameData> for RealTimeData {
    type Error = Error;

    fn try_from(value: FrameData) -> Result<Self, Self::Error> {
        macro_rules! parse {
            ($f:ident,$t:ty) => {
                paste::paste! {
                    match value.$f{
                        Value::Number(x) => x.[<as_ $t>](),
                        Value::String(x) => {
                            if x.eq("-"){
                                return Err(Error::MarketClosed);
                            }
                            x.parse().ok()
                        },
                        _ => return Err(Error::IncompatibleApi),
                    }.ok_or(Error::IncompatibleApi)?
                }
            };
        }

        let update_at = get_time_zone()
            .timestamp_millis_opt(parse!(update_at, i64))
            .unwrap();
        let recent_trading_date =
            NaiveDate::parse_from_str(&parse!(recent_trading_date, u64).to_string(), "%Y%m%d")
                .map_err(|_| Error::IncompatibleApi)?;

        Ok(RealTimeData {
            price: parse!(price, f64),
            volume: parse!(volume, u64),
            history_volume: parse!(history_volume, u64),
            update_at,
            recent_trading_date,
            name: value.name,
            opening_price: parse!(opening_price, f64),
            histroy_high_price: parse!(histroy_high_price, f64),
            histroy_low_price: parse!(histroy_low_price, f64),
            yesterday_closing_price: parse!(yesterday_closing_price, f64),
            limit_up_price: parse!(limit_up_price, f64),
            limit_down_price: parse!(limit_down_price, f64),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MsgArray<T> {
    #[serde(rename = "msgArray")]
    array: Vec<T>,
}
#[derive(Debug, Serialize, Deserialize)]
struct RawErrorMessage {
    stat: String,
}

/// newtype wrapper for the [`Client`] facilitating realtime data fetching
pub struct RealTime<'a>(&'a Client);

impl Client {
    /// get realtime API client
    pub fn realtime(&self) -> RealTime {
        RealTime(self)
    }
}

impl RealTime<'_> {
    /// fetch realtime data from TWSE
    pub async fn fetch(&self, stock: Stock) -> Result<RealTimeData, Error> {
        match self
            .fetch_raw(std::iter::once(stock))
            .await?
            .into_iter()
            .next()
        {
            Some(x) => x.try_into(),
            None => Err(Error::IncompatibleApi),
        }
    }
    /// fetch realtime data from TWSE in batch
    pub async fn fetch_batch(
        &self,
        stocks: impl Iterator<Item = Stock>,
    ) -> Result<Vec<RealTimeData>, Error> {
        self.fetch_raw(stocks)
            .await?
            .into_iter()
            .map(RealTimeData::try_from)
            .collect()
    }
    async fn fetch_raw(
        &self,
        stocks: impl Iterator<Item = Stock>,
    ) -> Result<Vec<FrameData>, Error> {
        let stocks = stocks
            .map(|stock| match stock {
                Stock::Live(id) => format!("tse_{}.tw", id),
                Stock::OverTheCounter(id) => format!("otc_{}.tw", id),
            })
            .collect::<Vec<String>>()
            .join("|");

        let res = self
            .0
             .0
            .get(ENDPOINT)
            .query(&[("ex_ch", stocks)])
            .send()
            .await?;

        if !res.status().is_success() {
            return Err(Error::RateLimitExceeded);
        }

        let body = res.bytes().await?;
        dbg!(&body);
        match serde_json::from_slice::<MsgArray<FrameData>>(&body) {
            Ok(x) => Ok(x.array),
            Err(_) => {
                let x: RawErrorMessage =
                    serde_json::from_slice(body.as_ref()).map_err(|_| Error::IncompatibleApi)?;
                Err(Error::ErrorStatMessage(x.stat))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Stock;

    #[tokio::test]
    async fn fetch() {
        let client = Client::new();
        match client.realtime().fetch(Stock::Live(2330)).await {
            Ok(x) => assert_eq!(x.name, "台積電"),
            Err(err) => match err {
                Error::MarketClosed => {}
                _ => panic!("unexpected error: {:?}", err),
            },
        };
    }
    #[tokio::test]
    async fn fetch_raw() {
        let client = Client::new();
        let data = client
            .realtime()
            .fetch_raw(std::iter::once(Stock::Live(2330)))
            .await
            .unwrap();
        dbg!(&data);
        assert_eq!(data.len(), 1);
        assert_eq!(data.get(0).unwrap().name, "台積電");
    }
}
