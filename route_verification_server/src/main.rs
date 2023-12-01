//! Launch Postgres and create `irv_server_test` before developing this.
use std::{env::args, fs::File, io::BufReader};

use anyhow::Result;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::{debug, error, warn};
use route_verification::{
    bgp::{Line, Report, ReportItem},
    ir::{AddrPfxRange, AutNum, Filter, Peering, RouteSet, RouteSetMember},
    lex::{expressions, io_wrapper_lines, lines_continued, rpsl_objects, RpslExpr},
};
use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult},
    types::ipnetwork::IpNetwork,
    Pool, Postgres,
};

const ONE_MEBIBYTE: usize = 1024 * 1024;
const ENOUGH: usize = 1000;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:postgres@localhost/irv_server_test")
        .await?;

    let args: Vec<String> = args().collect();
    match args[1].as_str() {
        "scan" => scan_db(&pool).await?,
        other => error!("Unknown command `{}`", other),
    }

    Ok(())
}

async fn scan_db(pool: &Pool<Postgres>) -> Result<()> {
    debug!("Opening RIPE.db.");
    let encoding = Encoding::for_label(b"latin1");
    let db = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(File::open("ripe.db")?),
    );

    let empty = "".to_string();
    let (mut n_mntner, mut n_route_obj) = (0, 0);
    // (mut n_as_set,
    // mut n_aut_num,
    // mut n_peering_set,
    // mut n_filter_set,
    // mut n_route_set)= (0, 0, 0, 0, 0, 0, 0);

    debug!("Checking through objects.");
    for obj in rpsl_objects(io_wrapper_lines(db)) {
        if obj.body.len() > ONE_MEBIBYTE {
            warn!(
                "Skipping {} object `{}` with a {}MiB body.",
                obj.class,
                obj.name,
                obj.body.len() / ONE_MEBIBYTE
            );
            continue;
        }

        match obj.class.as_str() {
            "mntner" => {
                if n_mntner > ENOUGH {
                    continue;
                }
                let matches = find_rpsl_object_fields(&obj.body, &["desc", "source"]);
                let (desc_s, source_s) = (&matches[0], &matches[1]);
                match insert_mntner_obj(
                    pool,
                    &obj.name,
                    &obj.body,
                    desc_s.get(0).unwrap_or(&empty),
                    source_s.get(0).unwrap_or(&empty),
                )
                .await
                {
                    Ok(_) => {
                        n_mntner += 1;
                        if n_mntner > ENOUGH && n_route_obj > ENOUGH {
                            break;
                        }
                    }

                    Err(_) => error!("Failed to insert mntner {}", &obj.name),
                }
            }
            "route" | "route6" => {
                if n_route_obj > ENOUGH {
                    continue;
                }
                let matches = find_rpsl_object_fields(&obj.body, &["origin"]);
                let origin = if let Ok(o) = matches[0][0].parse() {
                    o
                } else {
                    warn!(
                        "Failed to parse origin {} of route object {}",
                        &matches[0][0], obj.name
                    );
                    continue;
                };
                match insert_route_obj(pool, &obj.name, &obj.body, origin).await {
                    Ok(_) => {
                        n_route_obj += 1;
                        if n_mntner > ENOUGH && n_route_obj > ENOUGH {
                            break;
                        }
                    }
                    Err(_) => error!("Failed to insert route object {}", &obj.name),
                }
            }
            _ => (),
        }
    }
    debug!("Inserted enough mntner and route objects.");

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
    address_prefix: &str,
    body: &str,
    origin: i32,
) -> sqlx::Result<PgQueryResult> {
    insert_rpsl_obj(pool, address_prefix, body).await?;
    sqlx::query!(
        "insert into route_obj(address_prefix, origin, rpsl_obj_name) values ($1, $2, $3)",
        address_prefix as _,
        origin,
        address_prefix
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
) -> sqlx::Result<()> {
    sqlx::query!(
        "insert into rpsl_obj(rpsl_obj_name, body) values ($1, $2)",
        rpsl_obj_name,
        body
    )
    .execute(pool)
    .await?;

    let mnt_bys = &find_rpsl_object_fields(body, &["mnt-by"])[0];
    for mnt_by in mnt_bys {
        insert_rpsl_obj_mnt_by(pool, rpsl_obj_name, mnt_by).await?;
    }
    Ok(())
}

async fn insert_aut_num(
    pool: &Pool<Postgres>,
    rpsl_obj_name: &str,
    as_num: i32,
    as_name: &str,
    aut_num: &AutNum,
) -> Result<PgQueryResult> {
    insert_rpsl_obj(pool, rpsl_obj_name, &aut_num.body).await?;

    let imports_json = serde_json::to_value(&aut_num.imports)?;
    let exports_json = serde_json::to_value(&aut_num.exports)?;
    sqlx::query!(
        "insert into aut_num(as_num, as_name, imports, exports, rpsl_obj_name) values ($1, $2, $3, $4, $5)",
        as_num,
        as_name,
        imports_json,
        exports_json,
        rpsl_obj_name
    )
    .execute(pool)
    .await.map_err(Into::into)
}

async fn insert_observed_route(pool: &Pool<Postgres>, line: &Line) -> sqlx::Result<i32> {
    let raw_line = &line.raw;
    let prefix = line.compare.prefix;
    let address_prefix = IpNetwork::new(prefix.addr(), prefix.prefix_len())
        .expect("IpNet should be valid IpNetWork");

    let observed_route_id = sqlx::query!(
        r#"INSERT INTO observed_route(raw_line, address_prefix)
        VALUES ($1, $2)
        RETURNING observed_route_id"#,
        raw_line,
        address_prefix,
    )
    .fetch_one(pool)
    .await?
    .observed_route_id;

    if let Some(reports) = &line.report {
        for report in reports {
            _ = insert_exchange_report(pool, report, observed_route_id).await?;
        }
    }

    Ok(observed_route_id)
}

async fn insert_exchange_report(
    pool: &Pool<Postgres>,
    report: &Report,
    observed_route_id: i32,
) -> sqlx::Result<i32> {
    let (from_as, to_as, import, overall_type, items) = match report {
        Report::OkImport { from, to } => (*from as i32, *to as i32, true, "ok", None),
        Report::OkExport { from, to } => (*from as i32, *to as i32, false, "ok", None),
        Report::SkipImport { from, to, items } => {
            (*from as i32, *to as i32, true, "skip", Some(items))
        }
        Report::SkipExport { from, to, items } => {
            (*from as i32, *to as i32, false, "skip", Some(items))
        }
        Report::UnrecImport { from, to, items } => {
            (*from as i32, *to as i32, true, "unrecorded", Some(items))
        }
        Report::UnrecExport { from, to, items } => {
            (*from as i32, *to as i32, false, "unrecorded", Some(items))
        }
        Report::MehImport { from, to, items } => {
            (*from as i32, *to as i32, true, "special case", Some(items))
        }
        Report::MehExport { from, to, items } => {
            (*from as i32, *to as i32, false, "special case", Some(items))
        }
        Report::BadImport { from, to, items } => {
            (*from as i32, *to as i32, true, "bad", Some(items))
        }
        Report::BadExport { from, to, items } => {
            (*from as i32, *to as i32, false, "bad", Some(items))
        }
        report @ Report::AsPathPairWithSet { from: _, to: _ } => {
            warn!("Encountered {:?}", report);
            return Ok(-1);
        }
    };

    let report_id = sqlx::query!(
        r#"INSERT INTO exchange_report(from_as, to_as, import, overall_type, parent_observed_route)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING report_id"#,
        from_as,
        to_as,
        import,
        overall_type as _,
        observed_route_id,
    )
    .fetch_one(pool)
    .await?
    .report_id;

    if let Some(items) = items {
        for item in items {
            insert_report_item(pool, overall_type, item, report_id).await?;
        }
    }

    Ok(report_id)
}

async fn insert_report_item(
    pool: &Pool<Postgres>,
    category: &str,
    item: &ReportItem,
    exchange_report_id: i32,
) -> sqlx::Result<i32> {
    let (specific_case, str_content, num_content) = match item {
        ReportItem::SkipAsRegexWithTilde(s) => (Some("skip_regex_tilde"), Some(s), None),
        ReportItem::SkipAsRegexPathWithSet => (Some("skip_regex_with_set"), None, None),
        ReportItem::SkipCommunityCheckUnimplemented(_) => (Some("skip_community"), None, None),
        ReportItem::UnrecordedAutNum(num) => (Some("unrec_aut_num"), None, Some(*num as i32)),
        ReportItem::UnrecImportEmpty => (Some("unrec_import_empty"), None, None),
        ReportItem::UnrecExportEmpty => (Some("unrec_export_empty"), None, None),
        ReportItem::UnrecordedAsSet(s) => (Some("unrec_as_set"), Some(s), None),
        ReportItem::UnrecordedAsRoutes(num) => (Some("unrec_as_routes"), None, Some(*num as i32)),
        ReportItem::UnrecordedAsSetRoute(s) => (Some("unrec_as_set_route"), Some(s), None),
        ReportItem::UnrecordedSomeAsSetRoute(s) => (Some("unrec_some_as_set_route"), Some(s), None),
        ReportItem::UnrecordedRouteSet(s) => (Some("unrec_route_set"), Some(s), None),
        ReportItem::UnrecordedPeeringSet(s) => (Some("unrec_peering_set"), Some(s), None),
        ReportItem::UnrecordedFilterSet(s) => (Some("unrec_filter_set"), Some(s), None),
        ReportItem::SpecAsIsOriginButNoRoute(num) => (
            Some("spec_as_is_origin_but_no_route"),
            None,
            Some(*num as i32),
        ),
        ReportItem::SpecAsSetContainsOriginButNoRoute(s, num) => (
            Some("spec_as_set_contains_origin_but_no_route"),
            Some(s),
            Some(*num as i32),
        ),
        ReportItem::SpecExportCustomers => (Some("spec_export_customers"), None, None),
        ReportItem::SpecImportFromNeighbor => (Some("spec_import_from_neighbor"), None, None),
        ReportItem::SpecTier1Pair => (Some("spec_tier1_pair"), None, None),
        ReportItem::SpecImportPeerOIFPS => (Some("spec_import_peer_oifps"), None, None),
        ReportItem::SpecImportCustomerOIFPS => (Some("spec_import_customer_oifps"), None, None),
        ReportItem::SpecUphillTier1 => (Some("spec_uphill_tier1"), None, None),
        ReportItem::SpecUphill => (Some("spec_uphill"), None, None),
        ReportItem::MatchFilter => (Some("err_filter"), None, None),
        ReportItem::MatchFilterAsNum(num, _) => {
            (Some("err_filter_as_num"), None, Some(*num as i32))
        }
        ReportItem::MatchFilterAsSet(s, _) => (Some("err_filter_as_set"), Some(s), None),
        ReportItem::MatchFilterPrefixes => (Some("err_filter_prefixes"), None, None),
        ReportItem::MatchFilterRouteSet(s) => (Some("err_filter_route_set"), Some(s), None),
        ReportItem::MatchRemoteAsNum(num) => (Some("err_remote_as_num"), None, Some(*num as i32)),
        ReportItem::MatchRemoteAsSet(s) => (Some("err_remote_as_set"), Some(s), None),
        ReportItem::MatchExceptPeeringRight => (Some("err_except_peering_right"), None, None),
        ReportItem::MatchPeering => (Some("err_peering"), None, None),
        ReportItem::MatchRegex(s) => (Some("err_regex"), Some(s), None),
        ReportItem::RpslInvalidAsName(s) => (Some("rpsl_as_name"), Some(s), None),
        ReportItem::RpslInvalidFilter(s) => (Some("rpsl_filter"), Some(s), None),
        ReportItem::RpslInvalidAsRegex(s) => (Some("rpsl_regex"), Some(s), None),
        ReportItem::RpslUnknownFilter(s) => (Some("rpsl_unknown_filter"), Some(s), None),
        ReportItem::RecCheckFilter => (Some("recursion"), None, None),
        ReportItem::RecFilterRouteSet(s) => (Some("recursion"), Some(s), None),
        ReportItem::RecFilterRouteSetMember(_) => (Some("recursion"), None, None),
        ReportItem::RecFilterAsSet(s) => (Some("recursion"), Some(s), None),
        ReportItem::RecFilterAsName(_) => (Some("recursion"), None, None),
        ReportItem::RecFilterAnd => (Some("recursion"), None, None),
        ReportItem::RecFilterOr => (Some("recursion"), None, None),
        ReportItem::RecFilterNot => (Some("recursion"), None, None),
        ReportItem::RecCheckSetMember(s) => (Some("recursion"), Some(s), None),
        ReportItem::RecCheckRemoteAs => (Some("recursion"), None, None),
        ReportItem::RecRemoteAsName(_) => (Some("recursion"), None, None),
        ReportItem::RecRemoteAsSet(s) => (Some("recursion"), Some(s), None),
        ReportItem::RecRemotePeeringSet(s) => (Some("recursion"), Some(s), None),
        ReportItem::RecPeeringAnd => (Some("recursion"), None, None),
        ReportItem::RecPeeringOr => (Some("recursion"), None, None),
        ReportItem::RecPeeringExcept => (Some("recursion"), None, None),
    };

    // Insert the report item with its corresponding details
    let report_item_id = sqlx::query!(
        r#"INSERT INTO report_item(category, specific_case, str_content, num_content, parent_report)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING report_item_id"#,
        category as _,
        specific_case as _,
        str_content,
        num_content,
        exchange_report_id
    )
    .fetch_one(pool)
    .await?
    .report_item_id;

    Ok(report_item_id)
}

async fn insert_provide_customer(
    pool: &Pool<Postgres>,
    provider: i32,
    customer: i32,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        "INSERT INTO provide_customer(provider, customer) VALUES ($1, $2)",
        provider,
        customer
    )
    .execute(pool)
    .await
}

async fn insert_peer(
    pool: &Pool<Postgres>,
    peer_1: i32,
    peer_2: i32,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        "INSERT INTO peer(peer_1, peer_2) VALUES ($1, $2)",
        peer_1,
        peer_2
    )
    .execute(pool)
    .await
}

async fn insert_peering_set(
    pool: &Pool<Postgres>,
    peering_set_name: &str,
    peerings: &Vec<Peering>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO peering_set(peering_set_name, peerings) VALUES ($1, $2)",
        peering_set_name,
        serde_json::to_value(peerings)?
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_filter_set(
    pool: &Pool<Postgres>,
    filter_set_name: &str,
    filters: &Vec<Filter>,
) -> Result<()> {
    sqlx::query!(
        "INSERT INTO filter_set(filter_set_name, filters) VALUES ($1, $2)",
        filter_set_name,
        serde_json::to_value(filters)?
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_mbrs_by_ref(
    pool: &Pool<Postgres>,
    rpsl_obj_name: &str,
    mntner_name: &str,
) -> sqlx::Result<PgQueryResult> {
    sqlx::query!(
        "INSERT INTO mbrs_by_ref(rpsl_obj_name, mntner_name) VALUES ($1, $2)",
        rpsl_obj_name,
        mntner_name
    )
    .execute(pool)
    .await
}

async fn insert_route_set(
    pool: &Pool<Postgres>,
    route_set_name: &str,
    route_set: &RouteSet,
) -> sqlx::Result<()> {
    insert_rpsl_obj(pool, route_set_name, &route_set.body).await?;
    sqlx::query!(
        "INSERT INTO route_set(route_set_name) VALUES ($1)",
        route_set_name
    )
    .execute(pool)
    .await?;

    for member in &route_set.members {
        match member {
            RouteSetMember::RSRange(addr_pfx_range) => {
                insert_route_set_contains_address_prefix(pool, route_set_name, addr_pfx_range)
                    .await?;
            }
            RouteSetMember::NameOp(contained_set_name, _) => {
                insert_route_set_contains_set(pool, route_set_name, contained_set_name).await?;
            }
        }
    }

    Ok(())
}

async fn insert_route_set_contains_address_prefix(
    pool: &Pool<Postgres>,
    route_set_name: &str,
    addr_pfx_range: &AddrPfxRange,
) -> sqlx::Result<()> {
    let prefix = &addr_pfx_range.address_prefix;
    let address_prefix: IpNetwork = IpNetwork::new(prefix.addr(), prefix.prefix_len())
        .expect("IpNet should be valid IpNetWork");

    sqlx::query!(
        "INSERT INTO route_set_contains_address_prefix(route_set_name, address_prefix) VALUES ($1, $2)",
        route_set_name,
        address_prefix
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_route_set_contains_set(
    pool: &Pool<Postgres>,
    route_set_name: &str,
    contained_set_name: &str,
) -> sqlx::Result<()> {
    sqlx::query!(
        "INSERT INTO route_set_contains_set(route_set_name, contained_set) VALUES ($1, $2)",
        route_set_name,
        contained_set_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

fn find_rpsl_object_fields(body: &str, fields: &[&str]) -> Vec<Vec<String>> {
    let mut matches = vec![vec![]; fields.len()];
    for RpslExpr { key, expr } in expressions(lines_continued(body.lines())) {
        for (index, field) in fields.iter().enumerate() {
            if key == *field {
                matches[index].push(expr);
                break;
            }
        }
    }
    matches
}
