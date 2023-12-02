import os
import psycopg2
from dotenv import load_dotenv

from flask import Flask, render_template, request, url_for, redirect
# request, url_for, redirect

load_dotenv()

app = Flask(__name__)

# global variables to execute queries
# 1. for as_set_contains_num
AS_SET_NAME = (
    "select as_set_name from as_set_contains_num natural join autonomous_system where as_num = (%s)")


def get_db_connection():
    conn = psycopg2.connect(host='localhost',
                            database='flask_db',
                            user="postgres",
                            password="12345")
    return conn


@app.route('/')
def index():
    return render_template('index.html')


# write query page
# example for as_set_contains_num
@app.route('/query_page', methods=['GET', 'POST'])
def query_page():
    if request.method == 'POST':
        as_num = request.form.get('as_num', type=int)
        content_type = request.form.get('content_type')

        if as_num is not None and content_type:
            conn = get_db_connection()
            if content_type == 'rpsl':
                # Query and display RPSL information for the AS
                # Example: cur.execute("SELECT * FROM rpsl_obj WHERE as_num = %s", (as_num,))
                pass
            elif content_type == 'routes':
                # Query and display routes for the AS
                # Example: cur.execute("SELECT * FROM route_obj WHERE origin = %s", (as_num,))
                pass
            elif content_type == 'reports':
                # Query and display reports related to the AS
                # Example: cur.execute("SELECT * FROM exchange_report WHERE from_as = %s OR to_as = %s", (as_num, as_num))
                pass
            elif content_type == 'as_set':
                cur = conn.cursor()
                cur.execute(AS_SET_NAME, (as_num,))
                results = cur.fetchone()
                conn.close()

            # Fetch the results and close the connection

            return render_template('query_result.html',     =results)

    return render_template('query_page.html')


# write about page
@app.route('/About')
def About():
    return render_template('about.html')


@app.route('/create/', methods=('GET', 'POST'))
def create():
    if request.method == 'POST':
        title = request.form['title']
        author = request.form['author']
        pages_num = int(request.form['pages_num'])
        review = request.form['review']

        conn = get_db_connection()
        cur = conn.cursor()
        cur.execute('INSERT INTO books (title, author, pages_num, review)'
                    'VALUES (%s, %s, %s, %s)',
                    (title, author, pages_num, review))
        conn.commit()
        cur.close()
        conn.close()
        # return redirect(url_for('index'))

    return render_template('create.html')
