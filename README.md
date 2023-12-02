# Internet Route Verification Server

## Setup

Create a new PostgreSQL database `irv_server_test` and source the schema files at `./`:

For example, in `psql -h localhost`:

```sql
create database irv_server_test;
\c irv_server_test;
\i demo_v1.sql
\i trigger_only.sql
```

### Insertion to the database

This section should be done at `route_verification_server/`. You can do the following in parallel.

Move in `ripe.db` and run this to scan for maintainer objects and route objects.

```sh
cargo r --release scan
```

Move in the intermediate representation (IR) JSON files to `parsed_all/` and load them.
The JSON files can be generated following [instructions in internet_route_verification](https://github.com/SichangHe/internet_route_verification#produce-a-spread-parsed-dump-from-both-priority-and-backup-registries).

```sh
cargo r --release -- load
```

Move in the AS Relationship Dataset file `20230701.as-rel.bz2` and load them.

```sh
cargo r --release -- asrel
```

Make sure you have `bgpdump` installed.
Move in the MRT file `rib.20230619.2200.bz2`, generate 256 report on them, and load the reports into the database.

```sh
cargo r --release -- record
```
