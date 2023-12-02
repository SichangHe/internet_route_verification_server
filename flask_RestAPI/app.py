import os
import psycopg2
from dotenv import load_dotenv

from flask import Flask, render_template, request, url_for, redirect
# request, url_for, redirect

load_dotenv()

app = Flask(__name__)
# global queries
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
@app.route('/query_as_set')
def query_as_set():
    as_num = request.args.get('as_num', type=int)

    if as_num is not None:
        conn = get_db_connection()
        cur = conn.cursor()
        cur.execute(AS_SET_NAME, (as_num,))
        as_set_name = cur.fetchone()
        conn.close()

        if as_set_name:
            return render_template('query_as_set.html', as_set_name=as_set_name[0])
        else:
            return render_template('query_as_set.html', error_message="AS number not found.")
    else:
        return render_template('query_as_set.html', error_message="AS number not provided.")


# write about page
@app.route('/About')
def About():
    return render_template('about.html')


# @app.route('/create/', methods=('GET', 'POST'))
# def create():
#     if request.method == 'POST':
#         title = request.form['title']
#         author = request.form['author']
#         pages_num = int(request.form['pages_num'])
#         review = request.form['review']

#         conn = get_db_connection()
#         cur = conn.cursor()
#         cur.execute('INSERT INTO books (title, author, pages_num, review)'
#                     'VALUES (%s, %s, %s, %s)',
#                     (title, author, pages_num, review))
#         conn.commit()
#         cur.close()
#         conn.close()
#         # return redirect(url_for('index'))

#     return render_template('create.html')
