# beancount-price-fetcher

Fetches [price listings](https://beancount.github.io/docs/06_beancount_language_syntax.html#prices) for commodities to use in the [beancount](http://furius.ca/beancount/) plain text double entry accounting system. Makes use of the [OpenExchangeRates api](https://openexchangerates.org/), and can make asynchronous requests in parallel.

Install using `cargo install --git https://github.com/kellpossible/beancount-price-fetcher.git`.

## Usage

### `series` command

```text
Fetches a series of beancount price listings for commodities

USAGE:
    beancount-price-fetcher series [FLAGS] [OPTIONS] --app-id <ID> --start <DATE> --end <DATE> --commodities <COMMODITIES>... --base <COMMODITY>

FLAGS:
    -h, --help       Prints help information
    -d, --desc       Order the listings in descending order (by date)
    -V, --version    Prints version information

OPTIONS:
    -i, --app-id <ID>                     OpenExchangeRates App ID ( see https://openexchangerates.org/account/app-ids )
    -b, --base <COMMODITY>                Commodity to use as the reference/base in the beancount price listing
    -c, --commodities <COMMODITIES>...    Commodities to request exchange rates for (e.g AUD USD)
    -e, --end <DATE>                      End date in format YYYY-mm-dd, e.g. 2020-05-25
    -p, --parallel-requests <N>           Number of parallel network requests to use (when possible) [default: 2]
    -r, --rounding <DP>                   Number of decimal places to round to
    -s, --start <DATE>                    Start date in format YYYY-mm-dd, e.g. 2020-05-25
```

Example:

```bash
$ beancount-price-fetcher series --app-id {YOUR_APP_ID} --start 2020-01-01 --end 2020-01-05 --commodities NZD --base AUD -r 4 -d`
2020-01-05 price NZD 0.9583 AUD
2020-01-04 price NZD 0.9589 AUD
2020-01-03 price NZD 0.9589 AUD
2020-01-02 price NZD 0.9592 AUD
2020-01-01 price NZD 0.9595 AUD
```

### `usage` command

```text
Prints your api usage stats

USAGE:
    beancount-price-fetcher usage --app-id <ID>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -i, --app-id <ID>    OpenExchangeRates App ID ( see https://openexchangerates.org/account/app-ids )
```