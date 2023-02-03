# axum-api-server

## Why Axum

Quote from Discord: Although axum is less mature than actix-web it should have a brighter future because it is part of the tokio project and integrating with other libraries should be easier. It is also based on existing foundations used by other web servers, i.e. tower.
Using actix-web adds complications because it is using its own actix-rt runtime. It is based on tokio but it does its own thing with threads which may cause some incompatibilities with other projects. Libraries like sqlx and sea-orm have feature flags to use this runtime but most other projects typically just support tokio only. You can run actix-web under the tokio runtime but then you lose support for actix actors and websockets. I think it can be made to work but it isn't something that you really want people to deal with if they are just learning.

## Install Rust and global dependencies

`cargo install cargo-watch`

## Implement .env file from env.template

```
DATABASE_URL_EXAMPLE=protocol://username:password@host/database
DB_MYSQL_URL=
DB_POSTGRES_URL=
```

## Start Database

```
docker compose up -d --wait

docker compose logs database
```

## Generate Sea-orm entities

https://www.sea-ql.org/SeaORM/docs/generate-entity/sea-orm-cli/
[Note] Generate entity files if you change your database structure!

```
cargo install sea-orm-cli
sea-orm-cli generate entity -o src/entities -u YOUR_DATABASE_URI -o entity/src
```

## To Run in development mode

```bash
cargo watch -x run
```

## To Run in production mode

```bash
cargo run
```

## Make Docs for local dependency doc access

`cargo doc --open`
then open the generated html in a browser
