use std::{collections::HashMap, fmt};

use serde::{
    de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
    Deserialize, Serialize,
};

use super::YahooError;

#[derive(Deserialize, Debug)]
pub struct YResponse {
    pub chart: YChart,
}

impl YResponse {
    fn check_consistency(&self) -> Result<(), YahooError> {
        for stock in &self.chart.result {
            let n = stock.timestamp.len();
            if n == 0 {
                return Err(YahooError::EmptyDataSet);
            }
            let quote = &stock.indicators.quote[0];
            if quote.open.len() != n
                || quote.high.len() != n
                || quote.low.len() != n
                || quote.volume.len() != n
                || quote.close.len() != n
            {
                return Err(YahooError::DataInconsistency);
            }
            if let Some(ref adjclose) = stock.indicators.adjclose {
                if adjclose[0].adjclose.len() != n {
                    return Err(YahooError::DataInconsistency);
                }
            }
        }
        Ok(())
    }

    pub fn from_json(json: serde_json::Value) -> Result<YResponse, YahooError> {
        Ok(serde_json::from_value(json)?)
    }

    /// Return the latest valid quote
    pub fn last_quote(&self) -> Result<Quote, YahooError> {
        self.check_consistency()?;
        let stock = &self.chart.result[0];
        let n = stock.timestamp.len();
        for i in (0..n).rev() {
            let quote = stock.indicators.get_ith_quote(stock.timestamp[i], i);
            if quote.is_ok() {
                return quote;
            }
        }
        Err(YahooError::EmptyDataSet)
    }

    pub fn quotes(&self) -> Result<Vec<Quote>, YahooError> {
        self.check_consistency()?;
        let stock: &YQuoteBlock = &self.chart.result[0];
        let mut quotes = Vec::new();
        let n = stock.timestamp.len();
        for i in 0..n {
            let timestamp = stock.timestamp[i];
            let quote = stock.indicators.get_ith_quote(timestamp, i);
            if let Ok(q) = quote {
                quotes.push(q);
            }
        }
        Ok(quotes)
    }

    pub fn metadata(&self) -> Result<YMetaData, YahooError> {
        self.check_consistency()?;
        let stock = &self.chart.result[0];
        Ok(stock.meta.to_owned())
    }

    /// This method retrieves information about the splits that might have
    /// occured during the considered time period
    pub fn splits(&self) -> Result<Vec<Split>, YahooError> {
        self.check_consistency()?;
        let stock = &self.chart.result[0];
        if let Some(events) = &stock.events {
            if let Some(splits) = &events.splits {
                let mut data = splits.values().cloned().collect::<Vec<Split>>();
                data.sort_unstable_by_key(|d| d.date);
                return Ok(data);
            }
        }
        Ok(vec![])
    }
    /// This method retrieves information about the dividends that have
    /// been recorded during the considered time period.
    ///
    /// Note: Date is the ex-dividend date)
    pub fn dividends(&self) -> Result<Vec<Dividend>, YahooError> {
        self.check_consistency()?;
        let stock = &self.chart.result[0];
        if let Some(events) = &stock.events {
            if let Some(dividends) = &events.dividends {
                let mut data = dividends.values().cloned().collect::<Vec<Dividend>>();
                data.sort_unstable_by_key(|d| d.date);
                return Ok(data);
            }
        }
        Ok(vec![])
    }

    /// This method retrieves information about the capital gains that might have
    /// occured during the considered time period (available only for Mutual Funds)
    pub fn capital_gains(&self) -> Result<Vec<CapitalGain>, YahooError> {
        self.check_consistency()?;
        let stock = &self.chart.result[0];
        if let Some(events) = &stock.events {
            if let Some(capital_gain) = &events.capital_gains {
                let mut data = capital_gain.values().cloned().collect::<Vec<CapitalGain>>();
                data.sort_unstable_by_key(|d| d.date);
                return Ok(data);
            }
        }
        Ok(vec![])
    }
}

/// Struct for single quote
#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct Quote {
    pub timestamp: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub volume: u64,
    pub close: f64,
    pub adjclose: f64,
}

#[derive(Deserialize, Debug)]
pub struct YChart {
    pub result: Vec<YQuoteBlock>,
    pub error: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct YQuoteBlock {
    pub meta: YMetaData,
    pub timestamp: Vec<u64>,
    pub events: Option<EventsBlock>,
    pub indicators: QuoteBlock,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct YMetaData {
    pub currency: String,
    pub symbol: String,
    pub exchange_name: String,
    pub instrument_type: String,
    #[serde(default)]
    pub first_trade_date: Option<i32>,
    pub regular_market_time: u32,
    pub gmtoffset: i32,
    pub timezone: String,
    pub exchange_timezone_name: String,
    pub regular_market_price: f64,
    pub chart_previous_close: f64,
    pub previous_close: Option<f64>,
    #[serde(default)]
    pub scale: Option<i32>,
    pub price_hint: i32,
    pub current_trading_period: CurrentTradingPeriod,
    #[serde(default)]
    pub trading_periods: TradingPeriods,
    pub data_granularity: String,
    pub range: String,
    pub valid_ranges: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct TradingPeriods {
    pub pre: Option<Vec<Vec<PeriodInfo>>>,
    pub regular: Option<Vec<Vec<PeriodInfo>>>,
    pub post: Option<Vec<Vec<PeriodInfo>>>,
}

impl<'de> Deserialize<'de> for TradingPeriods {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Regular,
            Pre,
            Post,
        }

        struct TradingPeriodsVisitor;

        impl<'de> Visitor<'de> for TradingPeriodsVisitor {
            type Value = TradingPeriods;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct (or array) TradingPeriods")
            }

            fn visit_seq<V>(self, mut seq: V) -> Result<TradingPeriods, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let regular: Vec<PeriodInfo> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                Ok(TradingPeriods {
                    pre: None,
                    regular: Some(vec![regular]),
                    post: None,
                })
            }

            fn visit_map<V>(self, mut map: V) -> Result<TradingPeriods, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut pre = None;
                let mut post = None;
                let mut regular = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Pre => {
                            if pre.is_some() {
                                return Err(de::Error::duplicate_field("pre"));
                            }
                            pre = Some(map.next_value()?);
                        }
                        Field::Post => {
                            if post.is_some() {
                                return Err(de::Error::duplicate_field("post"));
                            }
                            post = Some(map.next_value()?);
                        }
                        Field::Regular => {
                            if regular.is_some() {
                                return Err(de::Error::duplicate_field("regular"));
                            }
                            regular = Some(map.next_value()?);
                        }
                    }
                }
                Ok(TradingPeriods { pre, post, regular })
            }
        }

        deserializer.deserialize_any(TradingPeriodsVisitor)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct CurrentTradingPeriod {
    pub pre: PeriodInfo,
    pub regular: PeriodInfo,
    pub post: PeriodInfo,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct PeriodInfo {
    pub timezone: String,
    pub start: u32,
    pub end: u32,
    pub gmtoffset: i32,
}

#[derive(Deserialize, Debug)]
pub struct QuoteBlock {
    quote: Vec<QuoteList>,
    #[serde(default)]
    adjclose: Option<Vec<AdjClose>>,
}

impl QuoteBlock {
    fn get_ith_quote(&self, timestamp: u64, i: usize) -> Result<Quote, YahooError> {
        let adjclose = match &self.adjclose {
            Some(adjclose) => adjclose[0].adjclose[i],
            None => None,
        };
        let quote = &self.quote[0];
        // reject if close is not set
        if quote.close[i].is_none() {
            return Err(YahooError::EmptyDataSet);
        }
        Ok(Quote {
            timestamp,
            open: quote.open[i].unwrap_or(0.0),
            high: quote.high[i].unwrap_or(0.0),
            low: quote.low[i].unwrap_or(0.0),
            volume: quote.volume[i].unwrap_or(0),
            close: quote.close[i].unwrap(),
            adjclose: adjclose.unwrap_or(0.0),
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct AdjClose {
    adjclose: Vec<Option<f64>>,
}

#[derive(Deserialize, Debug)]
pub struct QuoteList {
    pub volume: Vec<Option<u64>>,
    pub high: Vec<Option<f64>>,
    pub close: Vec<Option<f64>>,
    pub low: Vec<Option<f64>>,
    pub open: Vec<Option<f64>>,
}

#[derive(Deserialize, Debug)]
pub struct EventsBlock {
    pub splits: Option<HashMap<u64, Split>>,
    pub dividends: Option<HashMap<u64, Dividend>>,
    #[serde(rename = "capitalGains")]
    pub capital_gains: Option<HashMap<u64, CapitalGain>>,
}

/// This structure simply models a split that has occured.
#[derive(Deserialize, Debug, Clone)]
pub struct Split {
    /// This is the date (timestamp) when the split occured
    pub date: u64,
    /// Numerator of the split. For instance a 1:5 split means you get 5 share
    /// wherever you had one before the split. (Here the numerator is 1 and
    /// denom is 5). A reverse split is considered as nothing but a regular
    /// split with a numerator > denom.
    pub numerator: f64,
    /// Denominator of the split. For instance a 1:5 split means you get 5 share
    /// wherever you had one before the split. (Here the numerator is 1 and
    /// denom is 5). A reverse split is considered as nothing but a regular
    /// split with a numerator > denom.
    pub denominator: f64,
    /// A textual representation of the split.
    #[serde(rename = "splitRatio")]
    pub split_ratio: String,
}

/// This structure simply models a dividend which has been recorded.
#[derive(Deserialize, Debug, Clone)]
pub struct Dividend {
    /// This is the price of the dividend
    pub amount: f64,
    /// This is the ex-dividend date
    pub date: u64,
}

/// This structure simply models a capital gain which has been recorded.
#[derive(Deserialize, Debug, Clone)]
pub struct CapitalGain {
    /// This is the amount of capital gain distributed by the fund
    pub amount: f64,
    /// This is the recorded date of the capital gain
    pub date: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_period_info() {
        let period_info_json = r#"
        {
            "timezone": "EST",
            "start": 1705501800,
            "end": 1705525200,
            "gmtoffset": -18000
        }
        "#;
        let period_info_expected = PeriodInfo {
            timezone: "EST".to_string(),
            start: 1705501800,
            end: 1705525200,
            gmtoffset: -18000,
        };
        let period_info_deserialized: PeriodInfo = serde_json::from_str(period_info_json).unwrap();
        assert_eq!(&period_info_deserialized, &period_info_expected);
    }

    #[test]
    fn test_deserialize_trading_periods_simple() {
        let trading_periods_json = r#"
        [
            [
                {
                    "timezone": "EST",
                    "start": 1705501800,
                    "end": 1705525200,
                    "gmtoffset": -18000
                }

            ]
        ]
        "#;
        let trading_periods_expected = TradingPeriods {
            pre: None,
            regular: Some(vec![vec![PeriodInfo {
                timezone: "EST".to_string(),
                start: 1705501800,
                end: 1705525200,
                gmtoffset: -18000,
            }]]),
            post: None,
        };
        let trading_periods_deserialized: TradingPeriods =
            serde_json::from_str(trading_periods_json).unwrap();
        assert_eq!(&trading_periods_expected, &trading_periods_deserialized);
    }

    #[test]
    fn test_deserialize_trading_periods_complex_regular_only() {
        let trading_periods_json = r#"
        {
            "regular": [
              [
                {
                  "timezone": "EST",
                  "start": 1705501800,
                  "end": 1705525200,
                  "gmtoffset": -18000
                }
              ]
            ]
        }
       "#;
        let trading_periods_expected = TradingPeriods {
            pre: None,
            regular: Some(vec![vec![PeriodInfo {
                timezone: "EST".to_string(),
                start: 1705501800,
                end: 1705525200,
                gmtoffset: -18000,
            }]]),
            post: None,
        };
        let trading_periods_deserialized: TradingPeriods =
            serde_json::from_str(trading_periods_json).unwrap();
        assert_eq!(&trading_periods_expected, &trading_periods_deserialized);
    }

    #[test]
    fn test_deserialize_trading_periods_complex() {
        let trading_periods_json = r#"
        {
            "pre": [
              [
                {
                  "timezone": "EST",
                  "start": 1705482000,
                  "end": 1705501800,
                  "gmtoffset": -18000
                }
              ]
            ],
            "post": [
              [
                {
                  "timezone": "EST",
                  "start": 1705525200,
                  "end": 1705539600,
                  "gmtoffset": -18000
                }
              ]
            ],
            "regular": [
              [
                {
                  "timezone": "EST",
                  "start": 1705501800,
                  "end": 1705525200,
                  "gmtoffset": -18000
                }
              ]
            ]
        }
       "#;
        let trading_periods_expected = TradingPeriods {
            pre: Some(vec![vec![PeriodInfo {
                timezone: "EST".to_string(),
                start: 1705482000,
                end: 1705501800,
                gmtoffset: -18000,
            }]]),
            regular: Some(vec![vec![PeriodInfo {
                timezone: "EST".to_string(),
                start: 1705501800,
                end: 1705525200,
                gmtoffset: -18000,
            }]]),
            post: Some(vec![vec![PeriodInfo {
                timezone: "EST".to_string(),
                start: 1705525200,
                end: 1705539600,
                gmtoffset: -18000,
            }]]),
        };
        let trading_periods_deserialized: TradingPeriods =
            serde_json::from_str(trading_periods_json).unwrap();
        assert_eq!(&trading_periods_expected, &trading_periods_deserialized);
    }
}
