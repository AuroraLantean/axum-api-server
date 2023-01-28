use sqlx::MySqlPool;

/* Download from https://www.mysql.com/cloud/
connection name = db
Standard TCP/IP
localhost:3306
user = root    pw = aa
SSL: enabled with TLS_AES_256_GCM_SHA384
*/
pub async fn database_connection() -> Result<MySqlPool, sqlx::Error> {
    MySqlPool::connect("mysql://root:aa@localhost:3306").await
}
