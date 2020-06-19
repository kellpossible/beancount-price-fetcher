use chrono::{NaiveDateTime, Utc};
use commodity::{exchange_rate::ExchangeRate, CommodityTypeID};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Data from https://docs.openexchangerates.org/docs/latest-json and
/// https://docs.openexchangerates.org/docs/historical-json apis.
#[derive(Deserialize, Debug)]
pub struct OpenExchangeRate {
    timestamp: u32,
    base: CommodityTypeID,
    rates: BTreeMap<CommodityTypeID, Decimal>,
}

impl Into<ExchangeRate> for OpenExchangeRate {
    fn into(self) -> ExchangeRate {
        let date = Some(NaiveDateTime::from_timestamp(self.timestamp as i64, 0).date());

        ExchangeRate {
            date,
            obtained_datetime: Some(Utc::now()),
            base: Some(self.base),
            rates: self.rates,
        }
    }
}

/// Data from https://docs.openexchangerates.org/docs/usage-json
#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {
    pub data: UsageData,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AccessStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "access_restricted")]
    AccessRestricted,
}

/// The `data` field from https://docs.openexchangerates.org/docs/usage-json
#[derive(Serialize, Deserialize, Debug)]
pub struct UsageData {
    pub status: AccessStatus,
    pub plan: Plan,
    pub usage: UsageDataUsage,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlanFeatures {
    pub base: bool,
    pub symbols: bool,
    pub experimental: bool,
    #[serde(rename = "time-series")]
    pub time_series: bool,
    pub convert: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Plan {
    pub name: String,
    pub quota: String,
    pub update_frequency: String,
    pub features: PlanFeatures,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UsageDataUsage {
    pub requests: u32,
    pub requests_quota: u32,
    pub requests_remaining: u32,
    pub days_elapsed: u32,
    pub days_remaining: u32,
    pub daily_average: u32,
}
