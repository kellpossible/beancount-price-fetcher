use anyhow::anyhow;
use chrono::{Duration, NaiveDate, NaiveDateTime, Utc};
use clap::{App, Arg};
use commodity::{exchange_rate::ExchangeRate, CommodityTypeID};
use futures::{stream, StreamExt};
use reqwest::Client;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashSet},
    str::FromStr,
};

pub type AppID = String;

const API_URL: &'static str = "https://openexchangerates.org/api";

#[derive(Deserialize, Debug)]
struct OpenExchangeRate {
    timestamp: u32,
    base: CommodityTypeID,
    rates: BTreeMap<CommodityTypeID, Decimal>,
}

#[derive(Debug)]
pub struct TimeSeries {
    map: BTreeMap<NaiveDate, ExchangeRate>,
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

fn symbols_argument(includes: Vec<CommodityTypeID>) -> Option<String> {
    if !includes.is_empty() {
        let mut symbols = String::from("&symbols=");

        let includes_list = includes
            .iter()
            .map(|currency| currency.to_string())
            .collect::<Vec<String>>()
            .join(",");
        symbols.push_str(includes_list.as_str());
        Some(symbols)
    } else {
        None
    }
}

async fn get_day_json(
    client: &Client,
    app_id: &AppID,
    include: Option<Vec<CommodityTypeID>>,
    json: &str,
) -> anyhow::Result<ExchangeRate> {
    let mut url = format!(
        "{api_url}/{json}?app_id={app_id}",
        api_url = API_URL,
        app_id = app_id,
        json = json,
    );
    if let Some(includes) = include {
        if let Some(arg) = symbols_argument(includes) {
            url.push_str(arg.as_str());
        }
    }

    let rate: ExchangeRate = client
        .get(&url)
        .send()
        .await?
        .json::<OpenExchangeRate>()
        .await?
        .into();
    Ok(rate)
}

pub async fn get_latest(
    client: &Client,
    app_id: &AppID,
    include: Option<Vec<CommodityTypeID>>,
) -> anyhow::Result<ExchangeRate> {
    get_day_json(client, app_id, include, "latest.json").await
}

pub async fn get_historical(
    client: &Client,
    app_id: &AppID,
    date: &NaiveDate,
    include: Option<Vec<CommodityTypeID>>,
) -> anyhow::Result<ExchangeRate> {
    let date = format!("historical/{}.json", date.format("%Y-%m-%d").to_string());
    get_day_json(client, app_id, include, date.as_str()).await
}

pub async fn get_time_series_with_historical(
    client: &Client,
    app_id: &AppID,
    parallel_requests: usize,
    start: &NaiveDate,
    end: &NaiveDate,
    include: Option<Vec<CommodityTypeID>>,
) -> anyhow::Result<TimeSeries> {
    let mut series: BTreeMap<NaiveDate, ExchangeRate> = BTreeMap::new();
    let mut dates: Vec<NaiveDate> = Vec::new();

    let mut dt = start.clone();

    while &dt <= end {
        dates.push(dt);
        dt = dt + Duration::days(1);
    }

    let buffer = stream::iter(dates)
        .map(|date| {
            let include = include.clone();
            async move { get_historical(client, app_id, &date, include).await }
        })
        .buffer_unordered(parallel_requests);

    let results: Vec<anyhow::Result<ExchangeRate>> = buffer.collect().await;

    for result in results {
        match result {
            Ok(exchange_rate) => {
                series.insert(
                    exchange_rate.date.expect("expected date to be present"),
                    exchange_rate,
                );
            }
            Err(error) => return Err(error),
        }
    }

    Ok(TimeSeries { map: series })
}

// TODO: disabled because requires pro series plan, so I can't test right now.
// #[derive(Deserialize, Debug)]
// struct OpenExchangeTimeSeries {
//     start_date: NaiveDate,
//     end_date: NaiveDate,
// }

// async fn get_time_series(
//     app_id: &AppID,
//     start: &NaiveDate,
//     end: &NaiveDate,
//     include: Option<Vec<CommodityTypeID>>,
// ) -> anyhow::Result<OpenExchangeTimeSeries> {
//     let mut url = format!(
//         "{api_url}/time-series.json?app_id={app_id}&start={start}&end={end}",
//         api_url = API_URL,
//         app_id = app_id,
//         start = start.format("%Y-%m-%d").to_string(),
//         end = end.format("%Y-%m-%d").to_string()
//     );

//     if let Some(includes) = include {
//         if let Some(arg) = symbols_argument(includes) {
//             url.push_str(arg.as_str());
//         }
//     }

//     let series: OpenExchangeTimeSeries = reqwest::get(&url)
//         .await?
//         .json::<OpenExchangeTimeSeries>()
//         .await?;

//     Ok(series)
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let app = App::new("beancount-price-fetcher")
        .version("0.1")
        .author("Luke Frisken <l.frisken@gmail.com>")
        .about("Fetches beancount price listings for commodities")
        .subcommand(
            App::new("series")
                .about("Fetches a series of beancount price listings for commodities")
                .arg(
                    Arg::with_name("app-id")
                        .long("app-id")
                        .short('i')
                        .value_name("ID")
                        .about("OpenExchangeRates App ID ( see https://openexchangerates.org/account/app-ids )")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("start-date")
                        .long("start")
                        .short('s')
                        .value_name("DATE")
                        .about("Start date in format YYYY-mm-dd, e.g. 2020-05-25")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("end-date")
                        .long("end")
                        .short('e')
                        .value_name("DATE")
                        .about("End date in format YYYY-mm-dd, e.g. 2020-05-25")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("order-descending")
                        .long("desc")
                        .short('d')
                        .about("Order the listings in descending order (by date)")
                )
                .arg(
                    Arg::with_name("commodities")
                        .long("commodities")
                        .short('c')
                        .value_name("COMMODITIES")
                        .multiple(true)
                        .about("Commodities to request exchange rates for (e.g AUD USD)")
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("base")
                        .long("base")
                        .short('b')
                        .value_name("COMMODITY")
                        .about(
                            "Commodity to use as the reference/base in the beancount price listing",
                        )
                        .takes_value(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("parallel-requests")
                        .long("parallel-requests")
                        .short('p')
                        .value_name("N")
                        .about("Number of parallel network requests to use (when possible)")
                        .takes_value(true)
                        .default_value("2"),
                )
                .arg(
                    Arg::with_name("rounding")
                        .long("rounding")
                        .short('r')
                        .value_name("DP")
                        .about("Number of decimal places to round to")
                        .takes_value(true)
                ),
        );

    let matches = app.get_matches();

    if let Some(matches) = matches.subcommand_matches("series") {
        let app_id = matches
            .value_of("app-id")
            .expect("expected app-id to be specified")
            .to_string();
        let parallel_requests: usize = matches
            .value_of("parallel-requests")
            .expect("expected parallel-requests to be specified")
            .parse()
            .map_err(|err| anyhow!("unable to parse parallel-requests argument: {}", err))?;
        let start_date = NaiveDate::parse_from_str(
            matches
                .value_of("start-date")
                .expect("expected  start-date to be specified"),
            "%Y-%m-%d",
        )
        .map_err(|err| anyhow!("Unable to parse start-date: {}", err))?;
        let end_date = NaiveDate::parse_from_str(
            matches
                .value_of("end-date")
                .expect("expected start-date to be specified"),
            "%Y-%m-%d",
        )
        .map_err(|err| anyhow!("Unable to parse end-date: {}", err))?;
        let commodities: Vec<CommodityTypeID> = matches
            .values_of("commodities")
            .expect("expected commodities to be specified")
            .map(|commodity_str| {
                CommodityTypeID::from_str(commodity_str).expect("Unable to parse commodity id")
            })
            .collect();
        let base_commodity = CommodityTypeID::from_str(
            matches
                .value_of("base")
                .expect("expected base to be specified"),
        )
        .map_err(|err| anyhow!("Unable to parse base commodity id: {}", err))?;

        let mut request_commodities: HashSet<CommodityTypeID> = HashSet::new();

        for commodity in &commodities {
            request_commodities.insert(*commodity);
        }

        request_commodities.insert(base_commodity);

        let client = Client::new();

        let series = get_time_series_with_historical(
            &client,
            &app_id,
            parallel_requests,
            &start_date,
            &end_date,
            Some(request_commodities.into_iter().collect()),
        )
        .await?;

        for commodity in &commodities {
            let keys = series.map.keys();

            let keys: Box<dyn Iterator<Item = &NaiveDate>> = if matches.is_present("order-descending") {
                Box::new(keys.rev())
            } else {
                Box::new(keys.into_iter())
            };

            for key in keys {
                let exchange_rate = series.map.get(key).ok_or_else(|| format!("Exchange rate for date {} not present in the map", key)).unwrap();
                let mut rate_between = exchange_rate.rate_between(commodity, &base_commodity).map_err(|err| anyhow!("Unable to calculate the exchange rate between {} and {} because: {}", commodity, base_commodity, err))?.expect("unable to calculate the exchange rate between commodities");

                if let Some(rounding) = matches.value_of("rounding") {
                    let dp: u32 = rounding.parse().map_err(|err| anyhow!("Unable to parse rounding: {}", err))?;
                    rate_between = rate_between.round_dp(dp);
                }
                
                println!("{date} price {commodity} {rate} {base}", 
                    date = exchange_rate.date.unwrap().format("%Y-%m-%d"),
                    commodity = commodity,
                    rate = rate_between,
                    base = base_commodity,
                )
                
            }
        }
    }

    Ok(())
}
