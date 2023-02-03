/*
Choose database type: SQL or Postgres
-------------== SQL
sqlx = { version = "0.6.2", features = ["mysql", "runtime-tokio-native-tls"] }

Download from https://www.mysql.com/cloud/
connection name = connection_one
Standard TCP/IP
localhost:3306
user = root    pw = aa
SSL: enabled with TLS_AES_256_GCM_SHA384

Setup a database at SQL
https://www.mysqltutorial.org/mysql-create-database/

mysql://user:password@host/database_name

1046 (3D000): No database selected => add db at "mysql://root:aa@localhost:3306/db_name"

-------------== Postgres
sea-orm = { version = "0.10.7", features = ["sqlx-postgres", "runtime-tokio-rustls"] }

Use postgres database from Brooks tutorial
./zz.sh dbup

*/
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};

pub async fn connect_db(db_uri: &str) -> Result<DatabaseConnection, DbErr> {
    //db_uri should be protocol://username:password@host/database
    let mut opt = ConnectOptions::new(db_uri.to_owned());
    opt.max_connections(100);
    /*  .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info)
        .set_schema_search_path("my_schema".into()); // Setting default PostgreSQL schema
    */
    let db_conn = Database::connect(opt).await;
    db_conn
}

/*
use sqlx::MySqlPool;
pub async fn connect_db() -> Result<MySqlPool, sqlx::Error> {
    MySqlPool::connect("mysql://root:aa@localhost:3306/db1").await
} //127.0.0.1
*/
