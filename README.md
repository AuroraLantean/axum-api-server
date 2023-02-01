# axum-api-server

## Why Axum

Quote from Discord: Although axum is less mature than actix-web it should have a brighter future because it is part of the tokio project and integrating with other libraries should be easier. It is also based on existing foundations used by other web servers, i.e. tower.
Using actix-web adds complications because it is using its own actix-rt runtime. It is based on tokio but it does its own thing with threads which may cause some incompatibilities with other projects. Libraries like sqlx and sea-orm have feature flags to use this runtime but most other projects typically just support tokio only. You can run actix-web under the tokio runtime but then you lose support for actix actors and websockets. I think it can be made to work but it isn't something that you really want people to deal with if they are just learning.

## Install Rust and global dependencies

`cargo install cargo-watch`

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
