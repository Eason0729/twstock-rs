# twstock 台灣股市股票API🦀
    
[![crate.io](https://img.shields.io/crates/v/twstock.svg)](https://crates.io/crates/twstock)
[![master](https://github.com/Eason0729/twstock-rs/actions/workflows/master.yml/badge.svg)](https://github.com/Eason0729/twstock-rs/actions/workflows/master.yml)
[![codecov](https://codecov.io/github/Eason0729/twstock-rs/graph/badge.svg?token=RPYP79BLCZ)](https://codecov.io/github/Eason0729/twstock-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

API binding for fetching data from the [Taiwan Stock Exchange (TWSE)](https://www.twse.com.tw/zh/index.html)

## Getting started

```rust
use twstock::*;

#[tokio::main]
async fn main() {
    let client = Client::new();
    match client
        .realtime()
        .fetch(Stock {
            kind: StockKind::Live,
            code: 2330,
        })
        .await
    {
        Ok(x) => assert_eq!(x.name, "台積電"),
        Err(err) => match err {
            Error::MarketClosed => {}
            _ => panic!("unexpected error: {:?}", err),
        },
    };
}
```

## 特別感謝

[twstock(pypi)](https://github.com/mlouielu/twstock?tab=readme-ov-file): 給予API使用參考
[使用證卷交易所API爬取股票資訊(hackmd文章)](https://hackmd.io/@aaronlife/python-ex-stock-by-api)
