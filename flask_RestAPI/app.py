import psycopg
from dotenv import load_dotenv
from flask import Flask, jsonify, render_template
from psycopg.errors import InvalidTextRepresentation
from psycopg.rows import dict_row

OVERALL_REPORT_TYPES = ("ok", "skip", "unrecorded", "special_case", "bad")
REPORT_ITEM_TYPES = (
    "skip_regex_tilde",
    "skip_regex_with_set",
    "skip_community",
    "unrec_import_empty",
    "unrec_export_empty",
    "unrec_filter_set",
    "unrec_as_routes",
    "unrec_route_set",
    "unrec_as_set",
    "unrec_as_set_route",
    "unrec_some_as_set_route",
    "unrec_aut_num",
    "unrec_peering_set",
    "spec_uphill",
    "spec_uphill_tier1",
    "spec_tier1_pair",
    "spec_import_peer_oifps",
    "spec_import_customer_oifps",
    "spec_export_customers",
    "spec_import_from_neighbor",
    "spec_as_is_origin_but_no_route",
    "spec_as_set_contains_origin_but_no_route",
    "err_filter",
    "err_filter_as_num",
    "err_filter_as_set",
    "err_filter_prefixes",
    "err_filter_route_set",
    "err_remote_as_num",
    "err_remote_as_set",
    "err_except_peering_right",
    "err_peering",
    "err_regex",
    "rpsl_as_name",
    "rpsl_filter",
    "rpsl_regex",
    "rpsl_unknown_filter",
    "recursion",
)


def is_valid_overall_report_type(overall_report_type: str):
    return overall_report_type in OVERALL_REPORT_TYPES


def is_valid_report_item_type(report_item_type: str):
    return report_item_type in REPORT_ITEM_TYPES


load_dotenv()
app = Flask(__name__)

conn = psycopg.connect(f"dbname=irv_server_test", row_factory=dict_row)


@app.route("/rpsl_obj/<string:rpsl_obj_name>", methods=["GET"])
def get_rpsl_obj_by_name(rpsl_obj_name):
    return execute_one(
        "SELECT * FROM rpsl_obj WHERE rpsl_obj_name = %s",
        rpsl_obj_name,
    )


@app.route("/verification_reports/<int:observed_route_id>", methods=["GET"])
def get_verification_reports(observed_route_id):
    with conn.cursor() as cur:
        reports = cur.execute(
            "SELECT * FROM exchange_report WHERE parent_observed_route = %s",
            (observed_route_id,),
        ).fetchall()
    return jsonify(reports)


@app.route("/aut_num/<int:as_num>", methods=["GET"])
def get_aut_num(as_num: int):
    return execute_one(
        """
SELECT as_num, as_name, imports, exports FROM aut_num WHERE as_num = %s
""",
        as_num,
    )


@app.route("/as_for_overall_report_type/<string:overall_report_type>", methods=["GET"])
def get_as_for_overall_report_type(overall_report_type: str):
    """ASes that appear in at least one report with the given
    overall_report_type and the number of reports with that overall_report_type.
    """
    if not is_valid_overall_report_type(overall_report_type):
        return (
            jsonify(
                {
                    "input_error": "Invalid overall_report_type",
                    "possible_values": OVERALL_REPORT_TYPES,
                }
            ),
            400,
        )
    return execute_all(
        """
WITH filtered_exchange_report AS (
    SELECT * FROM exchange_report WHERE overall_type = %s
) SELECT aggregated_as.as_num, count(*) AS report_count FROM (
    (
        SELECT report_id, from_as AS as_num
        FROM filtered_exchange_report
        ORDER BY as_num
    ) UNION (
        SELECT report_id, to_as AS as_num
        FROM filtered_exchange_report
        ORDER BY as_num
    )
) AS aggregated_as
GROUP BY as_num
ORDER BY as_num
                """,
        overall_report_type,
    )


@app.route(
    "/route_for_overall_report_type_import_by_provider/<string:overall_report_type>",
    methods=["GET"],
)
def get_route_for_overall_report_type_import_by_provider(overall_report_type: str):
    """Routes that appear in at least one import report by a provider with
    the given overall_report_type and the number of reports with that
    overall_report_type.
    Potentially route leaks if bad.
    """
    if not is_valid_overall_report_type(overall_report_type):
        return (
            jsonify(
                {
                    "input_error": "Invalid overall_report_type",
                    "possible_values": OVERALL_REPORT_TYPES,
                }
            ),
            400,
        )
    # TODO: Paging.
    return execute_all(
        """
SELECT
    observed_route_id,
    raw_line,
    text(address_prefix),
    observed_route.recorded_time,
    count(*) AS report_count
FROM exchange_report
JOIN provide_customer ON from_as = customer AND to_as = provider
JOIN observed_route ON parent_observed_route = observed_route_id
WHERE overall_type = %s AND import = true
GROUP BY
    observed_route_id, raw_line, address_prefix, observed_route.recorded_time
                """,
        overall_report_type,
    )


@app.route("/as_for_report_item_type/<string:report_item_type>", methods=["GET"])
def get_as_for_report_item_type(report_item_type: str):
    """ASes that appear in at least one report item with the given
    report_item_type and the number of report items with that report_item_type.
    """
    if not is_valid_report_item_type(report_item_type):
        return (
            jsonify(
                {
                    "input_error": "Invalid report_item_type",
                    "possible_values": REPORT_ITEM_TYPES,
                }
            ),
            400,
        )
    return execute_all(
        """
WITH filtered_report_item AS (
    SELECT *
    FROM report_item JOIN exchange_report ON parent_report = report_id
    WHERE specific_case = %s
) SELECT aggregated_as.as_num, count(*) AS report_item_count FROM (
    (
        SELECT report_item_id, from_as AS as_num
        FROM filtered_report_item
        ORDER BY as_num
    ) UNION (
        SELECT report_item_id, to_as AS as_num
        FROM filtered_report_item
        ORDER BY as_num
    )
) AS aggregated_as
GROUP BY as_num
ORDER BY as_num
                """,
        report_item_type,
    )


@app.route("/for_overall_report_type/<string:overall_report_type>", methods=["GET"])
def get_for_overall_report_type(overall_report_type: str):
    """ASes, routes, reports, and report items for the given
    overall_report_type."""
    if not is_valid_overall_report_type(overall_report_type):
        return (
            jsonify(
                {
                    "input_error": "Invalid overall_report_type",
                    "possible_values": OVERALL_REPORT_TYPES,
                }
            ),
            400,
        )
    return execute_all(
        # TODO: Paging from user.
        """
SELECT
    e.from_as AS source_as,
    e.to_as AS destination_as,
    e.import,
    e.overall_type,
    e.recorded_time AS exchange_report_time,
    r.observed_route_id,
    r.raw_line,
    text(r.address_prefix) AS address_prefix,
    r.recorded_time AS observed_route_time,
    ri.category AS report_category,
    ri.specific_case AS report_specific_case,
    ri.str_content AS report_string_content,
    ri.num_content AS report_numeric_content,
    COUNT(*) OVER () AS total_items
FROM
    exchange_report e
JOIN
    observed_route r ON e.parent_observed_route = r.observed_route_id
LEFT JOIN
    report_item ri ON e.report_id = ri.parent_report
WHERE
    e.overall_type = %s
ORDER BY
    e.recorded_time
OFFSET
    0
LIMIT
    10
                """,
        overall_report_type,
    )


@app.route("/for_report_item_type/<string:report_item_type>", methods=["GET"])
def get_for_report_item_type(report_item_type: str):
    """ASes, routes, reports, and report items for the given
    report_item_type."""
    if not is_valid_report_item_type(report_item_type):
        return (
            jsonify(
                {
                    "input_error": "Invalid report_item_type",
                    "possible_values": REPORT_ITEM_TYPES,
                }
            ),
            400,
        )
    return execute_all(
        # TODO: Paging from user.
        """
SELECT
    e.from_as AS source_as,
    e.to_as AS destination_as,
    e.import,
    e.overall_type,
    e.recorded_time AS exchange_report_time,
    r.observed_route_id,
    r.raw_line,
    text(r.address_prefix) AS address_prefix,
    r.recorded_time AS observed_route_time,
    ri.category AS report_category,
    ri.specific_case AS report_specific_case,
    ri.str_content AS report_string_content,
    ri.num_content AS report_numeric_content,
    COUNT(*) OVER () AS total_items
FROM
    exchange_report e
JOIN
    observed_route r ON e.parent_observed_route = r.observed_route_id
JOIN
    report_item ri ON e.report_id = ri.parent_report
WHERE
    ri.specific_case = %s
ORDER BY
    exchange_report_time
OFFSET
    0
LIMIT
    10
                """,
        report_item_type,
    )


@app.route("/for_address_prefix/<string:address>/<int:prefix_length>", methods=["GET"])
def get_for_address_prefix(address: str, prefix_length: int):
    """Route, reports, and report items for the given address_prefix."""
    try:
        return execute_all(
            # TODO: Paging from user.
            # FIXME: Remove LIMIT 1.
            """
    SELECT
        e.report_id,
        e.from_as AS source_as,
        e.to_as AS destination_as,
        e.import,
        e.overall_type,
        e.recorded_time AS exchange_report_time,
        ri.report_item_id,
        ri.category AS report_category,
        ri.specific_case AS report_specific_case,
        ri.str_content AS report_string_content,
        ri.num_content AS report_numeric_content,
        COUNT(*) OVER () AS total_items
    FROM
        exchange_report e
    JOIN
        report_item ri ON e.report_id = ri.parent_report
    WHERE
        e.parent_observed_route = (SELECT observed_route_id FROM observed_route WHERE address_prefix = %s LIMIT 1)
    ORDER BY
        e.recorded_time
    OFFSET
        0
    LIMIT
        10
                    """,
            f"{address}/{prefix_length}",
        )
    except InvalidTextRepresentation:
        return (
            jsonify(
                {
                    "input_error": "Invalid address or prefix_length",
                    "address": address,
                    "prefix_length": prefix_length,
                }
            ),
            400,
        )


@app.route("/overall_report_type/<string:overall_report_type>", methods=["GET"])
def get_by_overall_report_type(overall_report_type):
    with conn.cursor() as cur:
        reports = cur.execute(
            "SELECT * FROM exchange_report WHERE overall_type = %s",
            (overall_report_type,),
        ).fetchall()
    return jsonify(reports)


@app.route("/report_for_as/<int:as_num>", methods=["GET"])
def get_report_for_as(as_num):
    return execute_all(
        """
SELECT * FROM (
    SELECT report_id
    FROM exchange_report
    JOIN autonomous_system ON exchange_report.from_as = autonomous_system.as_num
    WHERE as_num = %s
    union
    SELECT report_id
    FROM exchange_report
    JOIN autonomous_system ON exchange_report.to_as = autonomous_system.as_num
    WHERE as_num = %s
) AS as_and_report
natural JOIN exchange_report
ORDER BY report_id
                       """,
        as_num,
        as_num,
    )

@app.route("/route_for_as/<int:as_num>", methods=["GET"])
def get_route_for_as(as_num):
    return execute_all(
        """
select
    e.from_as as as_num,
    o.observed_route_id,
    o.raw_line,
    text(o.address_prefix) as address_prefix,
    o.recorded_time
from
    exchange_report as e
join
    observed_route as o
on e.parent_observed_route = o.observed_route_id
where e.from_as = %s
union
select
    e.to_as as as_num,
    o.observed_route_id,
    o.raw_line,
    text(o.address_prefix) as address_prefix,
    o.recorded_time
from
    exchange_report as e
join
    observed_route as o
on e.parent_observed_route = o.observed_route_id
where e.to_as = %s
                       """,
        as_num,
        as_num,
    )


@app.route("/")
def index():
    return render_template("index.html")


# write about page
@app.route("/About")
def About():
    return render_template("about.html")


def execute_one(sql, *args):
    with conn.cursor() as cur:
        entry = cur.execute(sql, (*args,)).fetchone()
        if entry:
            return jsonify(entry)
    return jsonify({"information": "Entry not found"}), 404


def execute_all(sql, *args):
    with conn.cursor() as cur:
        return jsonify(cur.execute(sql, (*args,)).fetchall())


if __name__ == "__main__":
    app.run(debug=True)
