import psycopg
from dotenv import load_dotenv
from flask import Flask, jsonify, render_template

load_dotenv()
app = Flask(__name__)

conn = psycopg.connect(f"dbname=irv_server_test")



@app.route("/rpsl_obj/<string:rpsl_obj_name>", methods=["GET"])
def get_rpsl_obj_by_name(rpsl_obj_name):
    with conn.cursor() as cur:
        entry = cur.execute(
            f"SELECT * FROM rpsl_obj WHERE rpsl_obj_name = %s", (rpsl_obj_name,)
        ).fetchone()
    if entry:
        return jsonify(entry)
    return jsonify({"message": "Entry not found"}), 404

@app.route("/verification_reports/<int:observed_route_id>", methods=["GET"])
def get_verification_reports(observed_route_id):
    with conn.cursor() as cur:
        reports = cur.execute(
            "SELECT * FROM exchange_report WHERE parent_observed_route = %s",
            (observed_route_id,),
        ).fetchall()
    return jsonify(reports)

@app.route("/as_info/<int:as_num>", methods=["GET"])
def get_as_info(as_num):
    with conn.cursor() as cur:
        rpsl_objs = cur.execute(
            "SELECT * FROM rpsl_obj WHERE as_num = %s", (as_num,)
        ).fetchall()
        routes = cur.execute(
            "SELECT * FROM route_obj WHERE origin = %s", (as_num,)
        ).fetchall()
    return jsonify({"rpsl_objs": rpsl_objs, "routes": routes})

@app.route("/overall_type/<string:overall_type>", methods=["GET"])
def get_by_overall_type(overall_type):
    with conn.cursor() as cur:
        reports = cur.execute(
            "SELECT * FROM exchange_report WHERE overall_type = %s", (overall_type,)
        ).fetchall()

    return jsonify(reports)

@app.route("/overall_type/<string:overall_type>", methods=["GET"])
def get_by_overall_type(overall_type):
    with conn.cursor() as cur:
        reports = cur.execute(
            "SELECT * FROM exchange_report WHERE overall_type = %s", (overall_type,)
        ).fetchall()
    return jsonify(reports)

@app.route("/overall_type/<string:overall_type>", methods=["GET"])
def get_by_overall_type(overall_type):
    with conn.cursor() as cur:
        reports = cur.execute(
            "SELECT * FROM exchange_report WHERE overall_type = %s", (overall_type,)
        ).fetchall()
    return jsonify(reports)










@app.route("/")
def index():
    return render_template("index.html")


# write about page
@app.route("/About")
def About():
    return render_template("about.html")


if __name__ == "__main__":
    app.run(debug=True)
