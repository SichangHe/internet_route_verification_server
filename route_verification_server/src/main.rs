//! Launch Postgres and create `irv_server_test` before developing this.
use std::{env::args, fs::File, io::BufReader};

use anyhow::Result;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;
use log::{debug, error, warn};
use route_verification::{
    as_rel::{AsRelDb, Relationship},
    bgp::{parse_mrt, Line, QueryIr, Report, ReportItem, Verbosity},
    ir::{AddrPfxRange, AutNum, FilterSet, Ir, PeeringSet, RouteSet, RouteSetMember},
    lex::{expressions, io_wrapper_lines, lines_continued, rpsl_objects, RpslExpr},
};
use sqlx::{
    postgres::{PgPoolOptions, PgQueryResult},
    types::ipnetwork::IpNetwork,
    Pool, Postgres,
};

const ONE_MEBIBYTE: usize = 1024 * 1024;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let pool = PgPoolOptions::new()
        .connect("postgres://postgres:postgres@localhost/irv_server_test")
        .await?;

    let args: Vec<String> = args().collect();
    match args[1].as_str() {
        "scan" => scan_db(&pool).await?,
        "load" => load_parsed(&pool).await?,
        "asrel" => as_relationship_db(&pool).await?,
        "record" => record_reports(&pool).await?,
        other => error!("Unknown command `{}`", other),
    }

    Ok(())
}

async fn record_reports(pool: &Pool<Postgres>) -> Result<()> {
    let mut n_observed_route = 0;
    debug!("Loading IR.");
    let db = AsRelDb::load_bz("20230701.as-rel.bz2")?;
    let parsed = Ir::pal_read("parsed_all")?;
    let query = QueryIr::from_ir_and_as_relationship(parsed, &db);

    debug!("Loading the MRT file.");
    let bgp_lines = parse_mrt("rib.20230619.2200.bz2")?;

    for mut line in bgp_lines {
        line.compare.verbosity = Verbosity::all_stats();
        line.check(&query);
        match insert_observed_route(pool, &line).await {
            Ok(_) => {
                n_observed_route += 1;
                if n_observed_route > 256 {
                    break;
                }
            }
            Err(why) => error!("Failed to insert observed route {:?}: {:?}", line, why),
        }
    }

    Ok(())
}

async fn as_relationship_db(pool: &Pool<Postgres>) -> Result<()> {
    let db = AsRelDb::load_bz("20230701.as-rel.bz2")?;

    for ((from, to), relationship) in &db.source2dest {
        match (from, to, relationship) {
            (provider, customer, Relationship::P2C) | (customer, provider, Relationship::C2P) => {
                debug!(
                    "Inserting provider-customer relationship {} -> {}",
                    from, to
                );
                match insert_provide_customer(pool, *provider as i32, *customer as i32).await {
                    Ok(_) => {}
                    Err(why) => error!(
                        "Failed to insert provider-customer relationship {} -> {}: {:?}",
                        from, to, why
                    ),
                }
            }
            (peer1, peer2, Relationship::P2P) => {
                debug!("Inserting peer relationship {} -> {}", from, to);
                match insert_peer(pool, *peer1 as i32, *peer2 as i32).await {
                    Ok(_) => {}
                    Err(why) => error!(
                        "Failed to insert peer relationship {} -> {}: {:?}",
                        from, to, why
                    ),
                }
            }
        }
    }

    Ok(())
}

async fn load_parsed(pool: &Pool<Postgres>) -> Result<()> {
    let empty = "".to_string();
    let Ir {
        aut_nums,
        as_sets,
        route_sets,
        peering_sets,
        filter_sets,
        as_routes: _,
    } = Ir::pal_read("parsed_all")?;

    for (num, aut_num) in aut_nums {
        debug!("Inserting aut-num {}", num);
        let rpsl_object_name = format!("AS{}", num);
        let as_num = num as i32;
        let as_names = &find_rpsl_object_fields(&aut_num.body, &["as-name"])[0];
        let as_name = as_names.get(0).unwrap_or(&empty);
        match insert_aut_num(pool, &rpsl_object_name, as_num, as_name, &aut_num).await {
            Ok(_) => {}
            Err(why) => error!("Failed to insert aut-num {}: {:?}", num, why),
        }
    }

    for (name, as_set) in as_sets {
        debug!("Inserting as-set {}", name);
        match insert_as_set(
            pool,
            &name,
            &as_set.body,
            as_set.is_any,
            &as_set.members,
            &as_set.set_members,
        )
        .await
        {
            Ok(_) => {}
            Err(why) => error!("Failed to insert as-set {}: {:?}", name, why),
        }
    }

    for (name, route_set) in route_sets {
        debug!("Inserting route-set {}", name);
        match insert_route_set(pool, &name, &route_set).await {
            Ok(_) => {}
            Err(why) => error!("Failed to insert route-set {}: {:?}", name, why),
        }
    }

    for (name, peering_set) in peering_sets {
        debug!("Inserting peering-set {}", name);
        match insert_peering_set(pool, &name, &peering_set).await {
            Ok(_) => {}
            Err(why) => error!("Failed to insert peering-set {}: {:?}", name, why),
        }
    }

    for (name, filter_set) in filter_sets {
        debug!("Inserting filter-set {}", name);
        match insert_filter_set(pool, &name, &filter_set).await {
            Ok(_) => {}
            Err(why) => error!("Failed to insert filter-set {}: {:?}", name, why),
        }
    }

    Ok(())
}

async fn scan_db(pool: &Pool<Postgres>) -> Result<()> {
    const ENOUGH: usize = 1000;
    debug!("Opening RIPE.db.");
    let encoding = Encoding::for_label(b"latin1");
    let db = BufReader::new(
        DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(File::open("ripe.db")?),
    );

    let empty = "".to_string();
    let (mut n_mntner, mut n_route_obj) = (0, 0);

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
                debug!("Inserting mntner {}", obj.name);
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

                    Err(why) => error!("Failed to insert mntner {}: {:?}", &obj.name, why),
                }
            }
            "route" | "route6" => {
                if n_route_obj > ENOUGH {
                    continue;
                }
                debug!("Inserting route object {}", obj.name);
                let matches = find_rpsl_object_fields(&obj.body, &["origin"]);
                let origin = if let Ok(o) = matches[0][0][2..].parse() {
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
                    Err(why) => error!("Failed to insert route object {}: {:?}", &obj.name, why),
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
) -> Result<()> {
    insert_rpsl_obj(pool, address_prefix, body).await?;
    sqlx::query!(
        "insert into route_obj(address_prefix, origin, rpsl_obj_name) values ($1, $2, $3)",
        address_prefix.parse::<IpNetwork>()?,
        origin,
        address_prefix
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_as_set(
    pool: &Pool<Postgres>,
    as_set_name: &str,
    body: &str,
    is_any: bool,
    num_members: &[u32],
    set_members: &[String],
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
            *num as i32
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

    for mbrs_by_ref in &find_rpsl_object_fields(body, &["mbrs-by-ref"])[0] {
        insert_mbrs_by_ref(pool, as_set_name, mbrs_by_ref).await?;
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
        Report::OkImport { from, to } => {
            (*from as i32, *to as i32, true, OverallReportType::Ok, None)
        }
        Report::OkExport { from, to } => {
            (*from as i32, *to as i32, false, OverallReportType::Ok, None)
        }
        Report::SkipImport { from, to, items } => (
            *from as i32,
            *to as i32,
            true,
            OverallReportType::Skip,
            Some(items),
        ),
        Report::SkipExport { from, to, items } => (
            *from as i32,
            *to as i32,
            false,
            OverallReportType::Skip,
            Some(items),
        ),
        Report::UnrecImport { from, to, items } => (
            *from as i32,
            *to as i32,
            true,
            OverallReportType::Unrecorded,
            Some(items),
        ),
        Report::UnrecExport { from, to, items } => (
            *from as i32,
            *to as i32,
            false,
            OverallReportType::Unrecorded,
            Some(items),
        ),
        Report::MehImport { from, to, items } => (
            *from as i32,
            *to as i32,
            true,
            OverallReportType::SpecialCase,
            Some(items),
        ),
        Report::MehExport { from, to, items } => (
            *from as i32,
            *to as i32,
            false,
            OverallReportType::SpecialCase,
            Some(items),
        ),
        Report::BadImport { from, to, items } => (
            *from as i32,
            *to as i32,
            true,
            OverallReportType::Bad,
            Some(items),
        ),
        Report::BadExport { from, to, items } => (
            *from as i32,
            *to as i32,
            false,
            OverallReportType::Bad,
            Some(items),
        ),
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
    category: OverallReportType,
    item: &ReportItem,
    exchange_report_id: i32,
) -> sqlx::Result<i32> {
    let (specific_case, str_content, num_content) = match item {
        ReportItem::SkipAsRegexWithTilde(s) => (ReportItemType::SkipRegexTilde, Some(s), None),
        ReportItem::SkipAsRegexPathWithSet => (ReportItemType::SkipRegexWithSet, None, None),
        ReportItem::SkipCommunityCheckUnimplemented(_) => {
            (ReportItemType::SkipCommunity, None, None)
        }
        ReportItem::UnrecordedAutNum(num) => (ReportItemType::UnrecAutNum, None, Some(*num as i32)),
        ReportItem::UnrecImportEmpty => (ReportItemType::UnrecImportEmpty, None, None),
        ReportItem::UnrecExportEmpty => (ReportItemType::UnrecExportEmpty, None, None),
        ReportItem::UnrecordedAsSet(s) => (ReportItemType::UnrecAsSet, Some(s), None),
        ReportItem::UnrecordedAsRoutes(num) => {
            (ReportItemType::UnrecAsRoutes, None, Some(*num as i32))
        }
        ReportItem::UnrecordedAsSetRoute(s) => (ReportItemType::UnrecAsSetRoute, Some(s), None),
        ReportItem::UnrecordedSomeAsSetRoute(s) => {
            (ReportItemType::UnrecSomeAsSetRoute, Some(s), None)
        }
        ReportItem::UnrecordedRouteSet(s) => (ReportItemType::UnrecRouteSet, Some(s), None),
        ReportItem::UnrecordedPeeringSet(s) => (ReportItemType::UnrecPeeringSet, Some(s), None),
        ReportItem::UnrecordedFilterSet(s) => (ReportItemType::UnrecFilterSet, Some(s), None),
        ReportItem::SpecAsIsOriginButNoRoute(num) => (
            ReportItemType::SpecAsIsOriginButNoRoute,
            None,
            Some(*num as i32),
        ),
        ReportItem::SpecAsSetContainsOriginButNoRoute(s, num) => (
            ReportItemType::SpecAsSetContainsOriginButNoRoute,
            Some(s),
            Some(*num as i32),
        ),
        ReportItem::SpecExportCustomers => (ReportItemType::SpecExportCustomers, None, None),
        ReportItem::SpecImportFromNeighbor => (ReportItemType::SpecImportFromNeighbor, None, None),
        ReportItem::SpecTier1Pair => (ReportItemType::SpecTier1Pair, None, None),
        ReportItem::SpecImportPeerOIFPS => (ReportItemType::SpecImportPeerOIFPS, None, None),
        ReportItem::SpecImportCustomerOIFPS => {
            (ReportItemType::SpecImportCustomerOIFPS, None, None)
        }
        ReportItem::SpecUphillTier1 => (ReportItemType::SpecUphillTier1, None, None),
        ReportItem::SpecUphill => (ReportItemType::SpecUphill, None, None),
        ReportItem::MatchFilter => (ReportItemType::ErrFilter, None, None),
        ReportItem::MatchFilterAsNum(num, _) => {
            (ReportItemType::ErrFilterAsNum, None, Some(*num as i32))
        }
        ReportItem::MatchFilterAsSet(s, _) => (ReportItemType::ErrFilterAsSet, Some(s), None),
        ReportItem::MatchFilterPrefixes => (ReportItemType::ErrFilterPrefixes, None, None),
        ReportItem::MatchFilterRouteSet(s) => (ReportItemType::ErrFilterRouteSet, Some(s), None),
        ReportItem::MatchRemoteAsNum(num) => {
            (ReportItemType::ErrRemoteAsNum, None, Some(*num as i32))
        }
        ReportItem::MatchRemoteAsSet(s) => (ReportItemType::ErrRemoteAsSet, Some(s), None),
        ReportItem::MatchExceptPeeringRight => (ReportItemType::ErrExceptPeeringRight, None, None),
        ReportItem::MatchPeering => (ReportItemType::ErrPeering, None, None),
        ReportItem::MatchRegex(s) => (ReportItemType::ErrRegex, Some(s), None),
        ReportItem::RpslInvalidAsName(s) => (ReportItemType::RpslAsName, Some(s), None),
        ReportItem::RpslInvalidFilter(s) => (ReportItemType::RpslFilter, Some(s), None),
        ReportItem::RpslInvalidAsRegex(s) => (ReportItemType::RpslRegex, Some(s), None),
        ReportItem::RpslUnknownFilter(s) => (ReportItemType::RpslUnknownFilter, Some(s), None),
        ReportItem::RecCheckFilter => (ReportItemType::Recursion, None, None),
        ReportItem::RecFilterRouteSet(s) => (ReportItemType::Recursion, Some(s), None),
        ReportItem::RecFilterRouteSetMember(_) => (ReportItemType::Recursion, None, None),
        ReportItem::RecFilterAsSet(s) => (ReportItemType::Recursion, Some(s), None),
        ReportItem::RecFilterAsName(_) => (ReportItemType::Recursion, None, None),
        ReportItem::RecFilterAnd => (ReportItemType::Recursion, None, None),
        ReportItem::RecFilterOr => (ReportItemType::Recursion, None, None),
        ReportItem::RecFilterNot => (ReportItemType::Recursion, None, None),
        ReportItem::RecCheckSetMember(s) => (ReportItemType::Recursion, Some(s), None),
        ReportItem::RecCheckRemoteAs => (ReportItemType::Recursion, None, None),
        ReportItem::RecRemoteAsName(_) => (ReportItemType::Recursion, None, None),
        ReportItem::RecRemoteAsSet(s) => (ReportItemType::Recursion, Some(s), None),
        ReportItem::RecRemotePeeringSet(s) => (ReportItemType::Recursion, Some(s), None),
        ReportItem::RecPeeringAnd => (ReportItemType::Recursion, None, None),
        ReportItem::RecPeeringOr => (ReportItemType::Recursion, None, None),
        ReportItem::RecPeeringExcept => (ReportItemType::Recursion, None, None),
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

#[derive(Copy, Clone, Debug, sqlx::Type)]
#[sqlx(type_name = "overall_report_type", rename_all = "snake_case")]
pub enum OverallReportType {
    Ok,
    Skip,
    Unrecorded,
    SpecialCase,
    Bad,
}

#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "report_item_type", rename_all = "snake_case")]
pub enum ReportItemType {
    SkipRegexTilde,
    SkipRegexWithSet,
    SkipCommunity,
    UnrecImportEmpty,
    UnrecExportEmpty,
    UnrecFilterSet,
    UnrecAsRoutes,
    UnrecRouteSet,
    UnrecAsSet,
    UnrecAsSetRoute,
    UnrecSomeAsSetRoute,
    UnrecAutNum,
    UnrecPeeringSet,
    SpecUphill,
    SpecUphillTier1,
    SpecTier1Pair,
    SpecImportPeerOIFPS,
    SpecImportCustomerOIFPS,
    SpecExportCustomers,
    SpecImportFromNeighbor,
    SpecAsIsOriginButNoRoute,
    SpecAsSetContainsOriginButNoRoute,
    ErrFilter,
    ErrFilterAsNum,
    ErrFilterAsSet,
    ErrFilterPrefixes,
    ErrFilterRouteSet,
    ErrRemoteAsNum,
    ErrRemoteAsSet,
    ErrExceptPeeringRight,
    ErrPeering,
    ErrRegex,
    RpslAsName,
    RpslFilter,
    RpslRegex,
    RpslUnknownFilter,
    Recursion,
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
    peering_set: &PeeringSet,
) -> Result<()> {
    insert_rpsl_obj(pool, peering_set_name, &peering_set.body).await?;
    sqlx::query!(
        "INSERT INTO peering_set(peering_set_name, peerings) VALUES ($1, $2)",
        peering_set_name,
        serde_json::to_value(&peering_set.peerings)?
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_filter_set(
    pool: &Pool<Postgres>,
    filter_set_name: &str,
    filter_set: &FilterSet,
) -> Result<()> {
    insert_rpsl_obj(pool, filter_set_name, &filter_set.body).await?;
    sqlx::query!(
        "INSERT INTO filter_set(filter_set_name, filters) VALUES ($1, $2)",
        filter_set_name,
        serde_json::to_value(&filter_set.filters)?
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

    for mbrs_by_ref in &find_rpsl_object_fields(&route_set.body, &["mbrs-by-ref"])[0] {
        insert_mbrs_by_ref(pool, route_set_name, mbrs_by_ref).await?;
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
