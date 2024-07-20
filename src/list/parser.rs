use crate::{Error, StockKind};
use chrono::NaiveDate;
use std::{borrow::Cow, ffi::CString};
use tl::*;

use encoding_rs::BIG5;

use super::StockInfo;

fn skip_on_table(text: &impl AsRef<str>) -> bool {
    !text
        .as_ref()
        .chars()
        .all(|c| c.is_whitespace() || c == '\n')
}

const FIELD_COUNT: usize = 7;
const REQUIRED_COUNT: usize = 5;

enum Column {
    CodeAndabbr,
    ReleaseDate,
    Industry,
    Kind,
    Unselected,
}

struct FieldMapper([Column; FIELD_COUNT]);

impl FieldMapper {
    fn new(fields: impl Iterator<Item = impl AsRef<str>>) -> Result<Self, Error> {
        let map = fields
            .filter(skip_on_table)
            .map(|field| match field.as_ref().trim() {
                "有價證券代號及名稱" => Column::CodeAndabbr,
                "上市日" => Column::ReleaseDate,
                "產業別" => Column::Industry,
                "市場別" => Column::Kind,
                _ => Column::Unselected,
            })
            .step_by(2)
            .collect::<Vec<_>>();
        let map = map.try_into().map_err(|_| Error::IncompatibleApi)?;
        Ok(Self(map))
    }
    fn map(&self, data: &[impl AsRef<str>; FIELD_COUNT]) -> Result<StockInfo, Error> {
        let mut result = StockInfo::default();

        for (value, key) in data.iter().zip(self.0.iter()) {
            let value = value.as_ref();
            match key {
                Column::CodeAndabbr => {
                    let segs = value.split_whitespace().collect::<Vec<_>>();
                    debug_assert!(segs.len() >= 2, "Invalid code and abbr: {}", value);
                    result.id.code = segs
                        .first()
                        .ok_or(Error::IncompatibleApi)?
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>()
                        .parse()
                        .map_err(|_| Error::IncompatibleApi)?;
                    result.abbr = segs.last().ok_or(Error::IncompatibleApi)?.to_string();
                }
                Column::ReleaseDate => {
                    result.release_date = NaiveDate::parse_from_str(value, "%Y/%m/%d")
                        .map_err(|_| Error::IncompatibleApi)?;
                }
                Column::Industry => {
                    result.industry = (*value).into();
                }
                Column::Kind => {
                    result.id.kind = match value {
                        "上市" => StockKind::Live,
                        "上櫃" => StockKind::OverTheCounter,
                        _ => StockKind::default(),
                    };
                }
                Column::Unselected => {}
            }
        }
        Ok(result)
    }
}

pub struct RawContent<'a>(pub &'a [u8]);

impl RawContent<'_> {
    /// parse the raw content to HTML and file table element
    pub fn parse(self) -> Result<Vec<StockInfo>, Error> {
        let raw_content = big5_to_utf8(self.0.to_vec());
        let dom =
            parse(&raw_content, ParserOptions::default()).map_err(|_| Error::IncompatibleApi)?;
        let parser = dom.parser();
        let mut entries = dom.query_selector("tr").ok_or(Error::IncompatibleApi)?;

        macro_rules! to_str_arr {
            ($e:expr) => {
                $e.get(parser)
                    .ok_or(Error::IncompatibleApi)?
                    .children()
                    .ok_or(Error::IncompatibleApi)?
                    .all(parser)
                    .iter()
                    .map(|x| x.inner_text(parser))
            };
        }
        let mapper =
            FieldMapper::new(to_str_arr!(entries.next().ok_or(Error::IncompatibleApi)?))?;

        let mut stocks = Vec::new();
        for entry in entries {
            let mut data = to_str_arr!(entry)
                .filter(skip_on_table)
                .step_by(2)
                .collect::<Vec<_>>();
            match data.len() < REQUIRED_COUNT {
                true => continue,
                false => data.resize_with(FIELD_COUNT, || Cow::Borrowed("")),
            }
            if let Ok(x) = mapper.map(&data.clone().try_into().unwrap()) {
                stocks.push(x)
            }
        }
        Ok(stocks)
    }
}

fn big5_to_utf8(raw: Vec<u8>) -> String {
    let c_string = CString::new(raw).unwrap();
    let c_str = c_string.as_c_str();

    BIG5.decode_without_bom_handling(c_str.to_bytes())
        .0
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse() {
        let raw = RawContent(include_bytes!("../../test/C_public.jsp.html.small"));
        let result = raw.parse().unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].abbr, "太設");
    }
    #[tokio::test]
    #[ignore = "This test is time-consuming"]
    async fn parse_large() {
        let raw = RawContent(include_bytes!("../../test/C_public.jsp.html.large"));
        for stock in raw.parse().unwrap() {
            print!("{}, ", stock.abbr);
        }
        println!("...");
    }
    #[test]
    fn test_big5_to_utf8() {
        let raw = include_bytes!("../../test/big5.test");
        let utf8 = big5_to_utf8(raw.to_vec());
        assert_eq!(utf8, "有價證券代號及名稱hello_world".to_string());
    }
}
