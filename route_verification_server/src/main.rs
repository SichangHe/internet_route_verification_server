/// Launch Postgres and create `irv_server_test` before developing this.
use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult},
    types::ipnetwork::IpNetwork,
    Pool, Postgres,
};

#[tokio::main]
async fn main() -> sqlx::Result<()> {
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

async fn insert_mntner_obj(
    pool: &Pool<Postgres>,
    mntner_name: &str,
    body: &str,
    desc_s: &str,
    source_s: &str,
) -> sqlx::Result<PgQueryResult> {
    insert_rpsl_obj(pool, mntner_name, body).await?;
    sqlx::query!(
        "insert into mntner_obj(mntner_name, desc_s, source_s) values ($1, $2, $3)",
        mntner_name,
        desc_s,
        source_s
    )
    .execute(pool)
    .await
}

async fn insert_route_obj(
    pool: &Pool<Postgres>,
    rpsl_obj_name: &str,
    body: &str,
    address_prefix: &IpNetwork,
    origin: i32,
) -> sqlx::Result<PgQueryResult> {
    insert_rpsl_obj(pool, rpsl_obj_name, body).await?;
    sqlx::query!(
        "insert into route_obj(address_prefix, origin, rpsl_obj_name) values ($1, $2, $3)",
        address_prefix,
        origin,
        rpsl_obj_name
    )
    .execute(pool)
    .await
}

async fn insert_as_set(
    pool: &Pool<Postgres>,
    as_set_name: &str,
    body: &str,
    is_any: bool,
    num_members: &[i32],
    set_members: &[&str],
) -> sqlx::Result<()> {
    insert_rpsl_obj(pool, as_set_name, body).await?;
    sqlx::query!(
        "insert into as_set(as_set_name, is_any) values ($1, $2)",
        as_set_name,
        is_any
    )
    .execute(pool)
    .await?;

    for num in num_members {
        sqlx::query!(
            "insert into as_set_contains_num(as_set_name, as_num) values ($1, $2)",
            as_set_name,
            num
        )
        .execute(pool)
        .await?;
    }

    for set in set_members {
        sqlx::query!(
            "insert into as_set_contains_set(as_set_name, contained_set) values ($1, $2)",
            as_set_name,
            set
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn insert_rpsl_obj_mnt_by(
    pool: &Pool<Postgres>,
    rpsl_obj_name: &str,
    mntner_name: &str,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        "insert into rpsl_obj_mnt_by(rpsl_obj_name, mntner_name) values ($1, $2)",
        rpsl_obj_name,
        mntner_name
    )
    .execute(pool)
    .await
}

async fn insert_rpsl_obj(
    pool: &Pool<Postgres>,
    rpsl_obj_name: &str,
    body: &str,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        "insert into rpsl_obj(rpsl_obj_name, body) values ($1, $2)",
        rpsl_obj_name,
        body
    )
    .execute(pool)
    .await
}

// TODO: aut_num.
// TODO: observed_route.
// TODO: exchange_report.
// TODO: report_item.
// TODO: provide_customer.
// TODO: peering_set.
// TODO: filter_set.
// TODO: peer.
// TODO: mbrs_by_ref.
// TODO: route_set.
