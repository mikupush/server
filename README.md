# Mikupush server

## Prerequisites

* PostgresSQL Server 17.6

## Configuration

### Environment variables

#### Server listen configuration

```text
MIKU_PUSH_SERVER_HOST=0.0.0.0
MIKU_PUSH_SERVER_PORT=8080
```

#### PostgresSQL connection configuration

```text
MIKU_PUSH_DATABASE_HOST=localhost
MIKU_PUSH_DATABASE_PORT=5432
MIKU_PUSH_DATABASE_NAME=postgres
MIKU_PUSH_DATABASE_USER=postgres
MIKU_PUSH_DATABASE_PASSWORD=postgres
```

#### Upload limits

Limit in bytes, if it is not defined, upload size will be unlimited.
```text
MIKU_PUSH_UPLOAD_MAX_SIZE=5000000000 # 5GB
```

## Run in debug mode

```sh
RUST_LOG=debug cargo run
```
