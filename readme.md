# twstock 台灣股市股票API🦀

從 [證券交易所 (TWSE)](https://www.twse.com.tw/zh/index.html) 下載股票資訊

## Getting started

```rust
use twstock::*;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let data = client.realtime().fetch(Stock::Live(2330)).await.unwrap();
    assert_eq!(data.name, "台積電");
}
```

## 特別感謝

[twstock(pypi)](https://github.com/mlouielu/twstock?tab=readme-ov-file): 給予API使用參考