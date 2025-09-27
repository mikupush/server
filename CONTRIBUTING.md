
# Contributing

## Prerequisites

* Docker
* Docker compose
* Diesel CLI

## Creating database migrations

For create database migrations, you must use Diesel CLI.
Make sure you have a PostgresSQL container running. You can use Docker Compose to run it.

```sh
docker compose up -d postgres
```

Install `cargo-binstall` and then diesel_cli if it is not installed.

👉 [Install cargo-binstall](https://github.com/cargo-bins/cargo-binstall?tab=readme-ov-file#installation)

```sh
cargo binstall diesel_cli
```

Create the `.env` file

```sh
cp .env.example .env
```

Create the database migration

```sh
# for example
diesel migration generate create_file_uploads
```

Run the database migrations

```sh
diesel migration run
```

## Tests

Before run tests, make sure you have set up the PostgresSQL container and ran the database migrations.

And ensure you have the `.env` file, if you want to have a different file for tests, you can create a `.env.test` file.

```sh
cargo test
```
