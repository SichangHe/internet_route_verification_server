# export PGCLIENTENCODING=UTF8
gunzip -c irv_server_dump.gz | psql -U postgres -d irv_server_test
# gunzip -c irv_server_dump.gz | psql -U postgres -d irv_server_test -f -


# psql -U postgres -d irv_server_test < irv_server_dump.gz
