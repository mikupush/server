# Mikupush server

## Usage
### Prerequisites

* PostgresSQL Server 17.6

### Configuration

#### Environment variables

##### Server listen configuration

```text
MIKU_PUSH_SERVER_HOST=0.0.0.0
MIKU_PUSH_SERVER_PORT=8080
```

##### PostgresSQL connection configuration

```text
MIKU_PUSH_DATABASE_HOST=localhost
MIKU_PUSH_DATABASE_PORT=5432
MIKU_PUSH_DATABASE_NAME=postgres
MIKU_PUSH_DATABASE_USER=postgres
MIKU_PUSH_DATABASE_PASSWORD=postgres
```

##### Upload limits

Limit in bytes, if it is not defined, upload size will be unlimited.
```text
MIKU_PUSH_UPLOAD_MAX_SIZE=5000000000 # 5GB
```

### Run in debug mode

```sh
RUST_LOG=debug cargo run
```

## Development

### Prerequisites

* Docker
* Docker compose

### Run Database Migrations

Create the PostgresSQL container first.

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
