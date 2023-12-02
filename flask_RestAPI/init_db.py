import os
import psycopg2

conn = psycopg2.connect(
    host="localhost",
    database="COMPSCI_DB",
    user="postgres",
    password="12345")

# Open a cursor to perform database operations
cur = conn.cursor()
cur.close()
conn.close()
