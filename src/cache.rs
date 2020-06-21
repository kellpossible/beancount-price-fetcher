use chrono::NaiveDate;
use commodity::exchange_rate::ExchangeRate;
use std::{collections::BTreeMap, path::{Path, PathBuf}, fs::File};
use bincode::deserialize_from;

pub trait ExchangeRateCache {
    fn get_exchange_rate(date: &NaiveDate) -> Option<&ExchangeRate>;
    fn put_exchange_rate(date: NaiveDate, exchange_rate: ExchangeRate) -> Option<ExchangeRate>;
    fn remove_exchange_rate(date: &NaiveDate) -> Option<ExchangeRate>;
}

pub struct FileExchangeRateCache {
    cache_file: PathBuf,
    exchange_rates: BTreeMap<NaiveDate, ExchangeRate>,
}

impl FileExchangeRateCache {
    pub fn open<F: Into<PathBuf>>(cache_file: F) -> anyhow::Result<Self> {
        let cache_file: PathBuf = cache_file.into();

        let exchange_rates = if cache_file.exists() && cache_file.is_file() {
            let file = File::open(&cache_file)?;
            deserialize_from(file)?
        } else {
            BTreeMap::new()
        };

        Ok(Self {
            cache_file,
            exchange_rates,
        })
    }
}

impl ExchangeRateCache for FileExchangeRateCache {
    fn get_exchange_rate(date: &NaiveDate) -> Option<&ExchangeRate> {
        todo!()
    }
    fn put_exchange_rate(date: NaiveDate, exchange_rate: ExchangeRate) -> Option<ExchangeRate> {
        // TODO: so I want to add thread that listens to updates to the cache,
        // and writes the cache file. It needs to be rate limited, in that,
        // multiple requests for writes (while writing), should only result in
        // the latest request being performed.
        // 
        // This buffer appears to have some of the behaviour needed,
        // in that the last item is replaced and it's almost lock free.
        // https://docs.rs/atomicring/1.2.5/atomicring/struct.AtomicRingBuffer.html
        // Use crossbeam-channel to send the requests, and use the queue
        // to ensure that requests are only performed when needed.
        // Alternatively https://docs.rs/single_value_channel/1.2.1/single_value_channel/ may provide
        // the required functionality in a single package!
        todo!()
    }
    fn remove_exchange_rate(date: &NaiveDate) -> Option<ExchangeRate> {
        todo!()
    }
}