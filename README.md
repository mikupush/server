# Mikupush server

## Usage
### Prerequisites

* PostgresSQL Server 17.6

## Configuration

MikuPush reads its configuration from a YAML file named `config.yaml` located in the project root (same directory as the binary when running locally). For containerized or production deployments, mount or provide a `config.yaml` file accordingly.

Every setting can be overridden using environment variables. When an environment variable is set, it takes precedence over the value in `config.yaml`. The available settings and their ENV alternatives are detailed below, mirroring `config.example.yaml`.

Configuration sources and precedence:
- You can configure everything using only environment variables; in that case, `config.yaml` is optional and can be omitted entirely.
- You can mix both: keep some values in `config.yaml` and provide others via environment variables.
- For each property, if the corresponding environment variable is set, it overrides the value from `config.yaml`.
- If a key is missing in `config.yaml` and no environment variable is provided, the built‑in default will be used (documented below and in `config.example.yaml`).

You can configure the server using either method:
- YAML file: edit/create `config.yaml` with the structure shown below.
- #### Environment variables export the variables shown below to override specific settings.

### Server (HTTP API)

#### YAML
```yaml
server:
  # IP address or hostname where the server will listen for connections.
  # Default: 0.0.0.0
  host: 0.0.0.0
  # Server TCP port.
  # Default: 8080
  port: 8080
```

#### Environment variables
```text
MIKU_PUSH_SERVER_HOST=0.0.0.0
MIKU_PUSH_SERVER_PORT=8080
```

### Database (PostgreSQL)

Note: the effective URL is built as:
"postgresql://postgres:postgres@localhost:5432/postgres"

#### YAML
```yaml
database:
  # Hostname or IP of the PostgreSQL server.
  # Default: localhost
  host: localhost
  # Port of the PostgreSQL server.
  # Default: 5432
  port: 5432
  # Name of the database to connect to.
  # Default: postgres
  database: postgres
  # Database user.
  # Default: postgres
  user: postgres
  # Database user password.
  # Default: postgres
  password: postgres
```
#### Environment variables
```text
MIKU_PUSH_DATABASE_HOST=localhost
MIKU_PUSH_DATABASE_PORT=5432
MIKU_PUSH_DATABASE_NAME=postgres
MIKU_PUSH_DATABASE_USER=postgres
MIKU_PUSH_DATABASE_PASSWORD=postgres
```

### File Upload

#### YAML
```yaml
upload:
  # Maximum allowed upload size in bytes.
  # If not set, or if set to "unlimited", there is no limit.
  # Example: 5000000000 for 5GB
  max_size: 5000000000
  # Directory where uploaded files are stored.
  # Can be a relative or absolute path.
  # Default: data
  directory: data
```
#### Environment variables
```text
# Number in bytes or the word "unlimited"
MIKU_PUSH_UPLOAD_MAX_SIZE=5000000000
# Relative or absolute path; default is "data"
MIKU_PUSH_UPLOAD_DIRECTORY=data
```

For a complete, commented template, see `config.example.yaml`. If a key is missing in `config.yaml` and there is no environment variable set, the application falls back to the documented default values.

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
