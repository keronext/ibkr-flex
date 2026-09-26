# ibkr-flex

`ibkr-flex` is a command-line tool for downloading and locally archiving Interactive Brokers Flex Query data.

It is designed for users who want to maintain their own local copy of their Interactive Brokers data in a structured and reproducible format.

`ibkr-flex` retrieves predefined Flex Queries from Interactive Brokers and stores the resulting CSV files locally.

## Principles

- Interactive Brokers data remains under the user's control.
- Flex reports are downloaded and stored on the local filesystem.
- Flex Queries must follow the configuration documented by this project.
- Downloaded files follow a predictable structure and naming convention.
- The resulting files can be archived or consumed by other tools and data-processing pipelines.

## IBKR Flex Query configuration

Before using `ibkr-flex`, the required Flex Queries must be created in the Interactive Brokers Client Portal.

Each Flex Query must contain the sections and columns expected by `ibkr-flex`.

The exact configuration required for each supported Flex Query will be documented in this repository.

Initial query types include:

- Cash
- Trades

Additional Flex Query types will be supported progressively.

> The complete list of required sections and columns is currently being defined.

## Usage

The CLI is currently under development.

The intended usage will look similar to:

```bash
ibkr-flex fetch cash \
    --query-id 1190415 \
    --from 2026-01-01 \
    --to 2026-01-31
```

`ibkr-flex` handles the communication with the IBKR Flex Web Service and stores the resulting CSV file locally according to its directory and naming conventions.

## Local storage

Flex reports are stored on the user's local filesystem.

The exact directory layout is still being defined. It will follow a predictable structure based on the Flex Query type and reporting period.

For example:

```text
ibkr/
├── cash/
│   ├── 2026-01.csv
│   └── 2026-02.csv
└── trades/
    ├── 2026-01.csv
    └── 2026-02.csv
```

The local archive belongs to the user and can be backed up, moved, or processed independently from `ibkr-flex`.

## Authentication

Access to the IBKR Flex Web Service requires a Flex Web Service token.

The token must not be passed directly as a command-line argument because command-line arguments may be exposed through shell history or process inspection.

The exact credential configuration mechanism will be documented before the first release.

## Security

Your Interactive Brokers Flex Web Service token is a secret.

Never:

- commit a Flex token to Git;
- embed a token in source code;
- include a token in logs;
- publish a token in an issue or bug report.

Users are responsible for storing their credentials securely.

## Scope

`ibkr-flex` focuses on retrieving and locally archiving Interactive Brokers Flex Query data.

It does not provide:

- portfolio management;
- accounting;
- portfolio analytics;
- broker reconciliation;
- remote data storage;
- user or account management.

The generated files remain independent and may be consumed by other applications or data-processing pipelines.

## Project status

> **Early development**

The CLI, local storage conventions, and required IBKR Flex Query configurations are currently being defined.

The interface and file conventions may change before the first stable release.

## Disclaimer

This project is an independent open-source project and is not affiliated with, endorsed by, or sponsored by Interactive Brokers LLC.

Interactive Brokers, IBKR, and related trademarks are the property of their respective owners.

Users are responsible for complying with Interactive Brokers' terms and requirements when accessing its services.

## License

Licensed under the Apache License, Version 2.0.

See [LICENSE](LICENSE) for details.
