use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/irv_server_test")
        .await?;

    const TEN: i64 = 10;
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(TEN)
        .fetch_one(&pool)
        .await?;

    assert_eq!(row.0, TEN);

    Ok(())
}
