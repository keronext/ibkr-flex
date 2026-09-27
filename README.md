# ibkr-flex

`ibkr-flex` is a command-line tool for downloading, locally archiving, and
validating Interactive Brokers Flex Query CSV reports.

It is designed for users who want a structured, reproducible local copy of
their IBKR data. Reports remain on the local filesystem and can be archived or
consumed by other tools and data-processing pipelines.

## Setup

Create a Flex Web Service token in the IBKR Client Portal, then configure it
locally:

```sh
cp .env.example .env
# Set FLEX_WEB_SERVICE_TOKEN in .env
```

The token is read only from `.env`; never pass it as a command-line argument.
Do not commit it, include it in source code or logs, or share it in an issue or
bug report.

## Flex Query configuration

Create the required Flex Queries in the IBKR Client Portal. Their query IDs
are stored locally in `flex-queries.toml`. Create it from the versioned
example, then replace each placeholder with the corresponding ID from the
Client Portal:

```sh
cp flex-queries.toml.example flex-queries.toml
```

`flex-queries.toml` is ignored by Git because it contains account-specific
configuration. Environment variables such as `FLEX_CASH` are not needed.

The currently configured query types are:

- `cash`
- `dividends_accruals`
- `interest_accruals`
- `lent`
- `nav_base`
- `options_paid`
- `pnl`
- `st_of_funds`
- `trades`

Each query type has a schema directory, for example
[`schemas/flex/cash/`](schemas/flex/cash/). Each JSON file in that directory
defines one version and records the expected CSV columns in their exact order.

Each schema includes a non-empty string `version_number` and validity period:
`date_start` is required, while `date_end: null` means that the version is
currently active. Dates use the `YYYY-MM-DD` format.
The validator selects the only schema whose validity period covers the report
month; overlapping periods are rejected.

## Usage

List the configured Flex Queries and their query IDs:

```sh
cargo run -- list flex
```

After a release build, the direct equivalent is:

```sh
./target/release/ibkr-flex-cli list flex
```

Fetches default to the last complete calendar month and write a report to
`downloads/<type>/YYYY-MM.csv`:

```sh
cargo run -- fetch cash --root ./downloads
```

### Build and run the binary

Build an optimized binary:

```sh
cargo build --release
```

Then run it directly, without Cargo:

```sh
./target/release/ibkr-flex-cli fetch cash --root ./downloads
```

Use `--date` for a year, month, or day:

```sh
cargo run -- fetch cash --date 2026 --root ./downloads
cargo run -- fetch cash --date 2026-08 --root ./downloads
cargo run -- fetch cash --date 2025-08-03 --root ./downloads
```

Use `--from` and `--to` for an arbitrary inclusive range:

```sh
cargo run -- fetch cash --from 2025-08-03 --to 2025-08-10 --root ./downloads
```

For a completed year, `--date 2025` requests the period from 2025-01-01 to
2025-12-31. Do not use `--date 2026` while 2026 is still in progress: it
would include future dates. Instead, request the available portion explicitly,
for example:

```sh
cargo run -- fetch cash --from 2026-01-01 --to 2026-09-26 --root ./downloads
```

The maximum range is 365 days, matching the IBKR limit. It may be reduced for
an invocation with `--max-days N` (between 1 and 365).

To use a Query ID that is not in `flex-queries.toml`:

```sh
cargo run -- fetch cash --query-id 12345678 --root ./downloads
```

Validate a downloaded report against its schema:

```sh
cargo run -- validate --type cash ./downloads/cash/2025-10.csv
```

The equivalent direct invocation is:

```sh
./target/release/ibkr-flex-cli validate --type cash ./downloads/cash/2025-10.csv
```

Validation warns when the report has no data rows. Schemas enforce the exact
column list in `schemas/flex/<type>/*.json`; column order is significant.
The CSV filename embeds its period: `YYYY.csv`, `YYYY-MM.csv`,
`YYYY-MM-DD.csv`, or `<start>_to_<end>.csv`. Its end date is used to select
the schema validity period.

## Local storage

Reports are kept locally using a predictable layout:

```text
downloads/
├── cash/
│   └── 2025-10.csv
└── trades/
    └── 2025-10.csv
```

The archive belongs to you and can be backed up, moved, or processed
independently from `ibkr-flex`.

## Scope

`ibkr-flex` retrieves and locally archives IBKR Flex Query data. It does not
provide portfolio management, accounting, analytics, reconciliation, remote
storage, or user/account management.

## Project status

> **Early development**

Interfaces, query configurations, and file conventions may change before the
first stable release.

## Disclaimer

This independent open-source project is not affiliated with, endorsed by, or
sponsored by Interactive Brokers LLC. Interactive Brokers, IBKR, and related
trademarks are the property of their respective owners. Users are responsible
for complying with Interactive Brokers' terms and requirements.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for
details.
