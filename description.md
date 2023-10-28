# Project Description

A REST API server for network operators to query routing policies and the verification reports for observed routes against these policies. The goal is to help operators verify the policies they publish and diagnose inconsistencies between policies and routes.

## Routing policy

Routing policies are retrieved from the Routing Policy Specification Language (RPSL) in the [Internet Route Registry (IRR)](https://www.irr.net/docs/list.html). Specified with the RPSL, each Autonomous System (AS) has its policies to accept or reject routes when importing or exporting them. Our service records these policies.

## Route verification report

We verify observed routes against the recorded policies to generate reports. Observed routes are from the [University of Oregon Route Views Archive Project](https://archive.routeviews.org/). Within each route, each import and export between two ASes is verified using their recorded policies; one report is generated for each import/export.

Each report contains both an overview and details of the verification. The overview ranges from "ok" to "bad". The details are lists of specific report items including the error types, skip reasons, and special cases.

We use [Internet Route Verification](https://github.com/SichangHe/internet_route_verification) to generate the reports.

## Motivation/ needs

- A structured way to query the IRR with a focus on routing policies.
- Storage and structured query for the large amount of observed routes and verification reports generated from them.

## User-facing functionality: query RPSL, routes, and reports

Users can query RPSL, routes, reports, and specific report items for a given AS, vice versa for specific use.
For example, query for the reports related to a specific AS; query for the ASes that have a specific type of report item; and query for routes related to a specific maintainer.

## Sketch of database design

```mermaid
erDiagram
im_export_report ||--o{ report_item : has
im_export_report {
    import bool
    overall_type enum
}
report_item {
    category enum
    specific_case enum
    str_content varchar(nullable)
    num_content int
}
path ||--|{ im_export_report : contains
path }|--|{ aut_num : goes-through
aut_num {
    as_num int
    as_name varchar
}
provider_customer }o--|{ aut_num : is-about
provider_customer {
    provider int
    customer int
}
observed_route ||--|| path : corresponds-to
observed_route }o--|| route_obj : corresponds-to
peer }o--|{ aut_num : are
peer {
    peer_1 int
    peer_2 int
}
route_obj {
    route varchar
    length int
    is_v6 bool
}
route_set }o--|{ route_obj: contains
route_obj }|--|| aut_num : origin
as_set ||--o{ aut_num : contains
as_set ||--o{ as_set : contains
rpsl_obj {
    name varchar
    body varchar
}
aut_num |o--|| rpsl_obj : is
as_set |o--|| rpsl_obj : is
peering_set |o--|| rpsl_obj : is
filter_set |o--|| rpsl_obj : is
route_set |o--|| rpsl_obj : is
route_obj |o--|| rpsl_obj : is
aut_num ||--|| mntner : by
as_set }o--o{ mntner: mbrs-by-ref
route_set }o--o{ mntner: mbrs-by-ref
```

- All objects have `time_updated`.
