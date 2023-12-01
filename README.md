# Internet Route Verification Server

## Setup

Create a new PostgreSQL database `irv_server_test` and source the schema files at `./`:

```sql
\i demo_v1.sql
\i trigger_only.sql
```

At `route_verification_server/`, move in `ripe.db` and run this to scan for maintainer objects and route objects.

```sh
cargo r --release scan
```
