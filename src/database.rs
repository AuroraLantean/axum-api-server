use sqlx::MySqlPool;

/* Download from https://www.mysql.com/cloud/
connection name = connection_one
Standard TCP/IP
localhost:3306
user = root    pw = aa
SSL: enabled with TLS_AES_256_GCM_SHA384

Setup a database at SQL
https://www.mysqltutorial.org/mysql-create-database/

mysql://user:password@host/database_name

1046 (3D000): No database selected => add db at "mysql://root:aa@localhost:3306/db_name"
*/
pub async fn database_connection() -> Result<MySqlPool, sqlx::Error> {
    MySqlPool::connect("mysql://root:aa@localhost:3306/db1").await
} //127.0.0.1
