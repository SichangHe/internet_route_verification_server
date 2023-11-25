use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost/irv_server_test")
        .await?;

    const BLAH: &str = "blah";
    let row = sqlx::query!("SELECT $1 as str", BLAH)
        .fetch_one(&pool)
        .await?;

    assert_eq!(row.str, Some(BLAH.to_owned()));

    Ok(())
}
