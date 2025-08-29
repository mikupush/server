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

#### PostgreSQL connection configuration

```text
MIKU_PUSH_DATABASE_URL=postgres://localhost:5432/postgres
MIKU_PUSH_DATABASE_USER=postgres
MIKU_PUSH_DATABASE_PASSWORD=postgres
```

## Run in debug mode

```sh
RUST_LOG=debug cargo run
```
