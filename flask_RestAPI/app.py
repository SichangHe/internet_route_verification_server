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


@app.route("/")
def index():
    return render_template("index.html")


# write about page
@app.route("/About")
def About():
    return render_template("about.html")


if __name__ == "__main__":
    app.run(debug=True)
