import psycopg
from dotenv import load_dotenv
from flask import Flask, jsonify, render_template
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


@app.route("/overall_report_type/<string:overall_report_type>", methods=["GET"])
def get_by_overall_report_type(overall_report_type):
    with conn.cursor() as cur:
        reports = cur.execute(
            "SELECT * FROM exchange_report WHERE overall_type = %s",
            (overall_report_type,),
        ).fetchall()
    return jsonify(reports)


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
