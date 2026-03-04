
# Contributing

## Prerequisites

* Rust 1.89 or later
* Node.js 22 or later
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

## Scripts and Web frontend

To run scripts and build Web frontend, you have to install all node dependencies.

```sh
npm install
```

> ℹ️ **NOTE**
> If you want to install a dependency for a script, you must install it as dev dependency.
> 
> For example if you want to install `glob` you must run:
>
> ```sh
> npm install --save-dev glob
> ```

Then you can run vite dev server for debug the Web frontend.

```sh
npm run dev
```

Or build the Web frontend for production.

```sh
npm run build
```

Or run a script in `package.json`. If it is not in `package.json` you can run it 
from the `scripts` directory. Not all scripts are written in TypeScript.
