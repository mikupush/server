# Mikupush server

## Installation

### Prerequisites

* PostgresSQL Server 17.6

### Bare metal

TODO

### Docker

The official Docker image is `mikupush/server`. The container listens on port 8080 by default. For configuration details and precedence (YAML vs environment variables), see the [Configuration section](#configuration).

Remember to create the `config.yaml` file if you don't have it. You can use the `config.example.yaml` file as a template.

```sh
curl -L https://raw.githubusercontent.com/mikupush/server/refs/heads/main/config.example.yaml -o config.yaml
```

Then you can run the container:

```sh
docker run -d \
  --name mikupush-server \
  -p 8080:8080 \
  -v "./data:/data" \
  -v "./config.yaml:/config.yaml:ro" \
  mikupush/server:latest
```

You can use environment variables instead of `config.yaml`, or use both using environment variables for overrides.
Refer to the [Configuration section](#configuration) for how to provide settings via `config.yaml` or environment variables.

#### Docker Compose

You can use Docker Compose to run the server in a container.
Write a `docker-compose.yml` file like this:

```yaml
services:
  server:
    image: mikupush/server:latest
    ports:
      - "8080:8080"
    volumes:
      - ./data:/data
      - ./config.yaml:/config.yaml:ro
```

## Usage

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

### Logging

#### YAML
```yaml
log:
  # Log level. Possible values: trace, debug, info, warn, error
  # Default: info
  level: info
  # Log output. Possible values: console, file
  # Default: console
  output: console
  # Log file name, used when output is file
  file: server.log
  # Log directory, used when output is file
  # Platform-specific default if not specified:
  # - Linux: /var/log/io.mikupush.server
  # - macOS: /usr/local/var/log/io.mikupush.server
  # - Windows: %LOCALAPPDATA%\io.mikupush.server\logs
  directory: /var/log/io.mikupush.server
  # JSON log format
  # Default: false
  json: false
```

#### Environment variables
```text
MIKU_PUSH_LOG_LEVEL=info
MIKU_PUSH_LOG_OUTPUT=console
# File name when output is file
MIKU_PUSH_LOG_FILE=server.log
# Directory when output is file
MIKU_PUSH_LOG_DIRECTORY=/var/log/io.mikupush.server
MIKU_PUSH_LOG_JSON=false
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
* Diesel CLI

### Creating database migrations

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
