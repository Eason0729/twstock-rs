# twstock 台灣股市股票API🦀

[![master](https://github.com/Eason0729/twstock-rs/actions/workflows/master.yml/badge.svg)](https://github.com/Eason0729/twstock-rs/actions/workflows/master.yml)
[![codecov](https://codecov.io/github/Eason0729/twstock-rs/graph/badge.svg?token=RPYP79BLCZ)](https://codecov.io/github/Eason0729/twstock-rs)
[![License Badge]](./LICENSE)

從 [證券交易所 (TWSE)](https://www.twse.com.tw/zh/index.html) 下載股票資訊

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