# Entity-Relationship (ER) Model

An ER model representing this group project. Visualization:
erDiagram
%% entity sets
    route_set }o--|{ route_obj: contains
    route_obj }|--|| aut_num : origin
    as_set ||--o{ aut_num : contains
    as_set ||--o{ as_set : contains 
    route_obj ||--o{ mntner_obj : contains
    rpsl_obj {
        text rpsl_obj_name PK
        text body
        string timestamp
    }
    maintainer ||--|{mntner_obj : contains
    maintainer {
        text mntner_name PK
    }
    mntner_obj {
        text mntner_name PK
        text desc_s
        text source_s
    }
    autonomous_system||--||exchange_report : allow
    autonomous_system{
        int as_num PK
    }
    path ||--|{ exchange_report : contains
    path }|--|{ aut_num : goes-through
    aut_num{
        int as_num PK
        text as_name
        json imports
        json exports
        text rpsl_obj_name
    }
    observed_route ||--|| path : corresponds-to
    observed_route }o--|| route_obj : corresponds-to
    peer }o--|{ aut_num : are
    observed_route{
        serial observed_route_id PK
        text raw_line
        inet address_prefix
        timestamp recorded_time
    }
    exchange_report ||--o{ report_item : has
    exchange_report{
        serial report_id PK
        int from_as
        int to_as
        bool import
        overall_report_type overall_type
        int parent_observed_route
        timestamp recorded_time
    }
    report_item ||--||exchange_report : allow
    report_item{
        serial report_item_id PK
        overall_report_type category
        report_item_type specific_case
        text str_content
        int num_content
        int parent_report
    }
    provide_customer }o--|{ aut_num : is-about
    provide_customer{
        int provider
        int customer
        timestamp recorded_time
    }
    peering_set{
        text peering_set_name
        json peerings
    }
    filter_set{
        text filter_set_name
        json filters
    }
    route_obj{
        inet address_prefix PK
        int origin 
        text rpsl_obj_name
    }
    peer{
        int peer_1
        int peer_2
        timestamp recorded_time
    }
    as_set{
        text as_set_name
        boolean is_any
    }
    as_set_contains_num ||--o{  as_set : contains
    as_set_contains_num ||--o{  autonomous_system : contain
    as_set_contains_num{
        text as_set_name
        int as_num
    }
    as_set_contains_set ||--o{ as_set : contains
    as_set_contains_set{
        text as_set_name
        text contained_set
    }
    mbrs_by_ref{
        text rpsl_obj_name
        text mntner_name
    }

    route_set{
        text route_set_name
    }
    route_set_contains_num ||--o{  route_set : contains
    route_set_contains_num ||--o{  autonomous_system : contains
    route_set_contains_num{
        text route_set_name
        int as_num
    }
    route_set_contains_set{
        text route_set_name
        text contained_set
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

Explanation and discussion for designed entity sets and relationship sets:

`rpsl_obj`, `maintainer`, and `autonomous_system` are basic entity sets
representing RPSL objects, maintainer objects, and AS objects.
The `rpsl_obj` has a multi-valued attribute `mnt_by` to store which maintainers
an RPSL object is maintained by.
`aut_num` is a weak entity set identified by its `autonomous_system` and
it is also related to `rpsl_obj` one-to-one.
Each `observed_route` has a multi-value composite attribute `exchange_report`,
and each `exchange_report` has a multi-value composite attribute `report_item`.
`autonomous_system` can have many-to-many provider-customer relationship with
each other in the `provider_customer` relationship set,
or they can have many-to-many peer-to-peer relationship with each other in
the `peer` relationship set.
`peering_set`, `filter_set`, `as_set`, and `route_set` are weak entity sets
identified by its `autonomous_system`.
`as_set` has a multi-value attribute `mbrs_by_ref` to refer to
its members by being referenced from the ASes,
a multi-value attribute `num_members` for specifying AS members,
and a multi-value attribute `set_members` for specifying AS Set members.
Same for `route_set`.

# Relational Model

Converted relational database model, visualized in a schema diagram:

Explanation for each step carried out:

The primary key of `rpsl_obj`, `maintainer`, `autonomous_system` in
the entity set are used as their primary key in the relational model.
Composite attributes `exchange_report` and `report_item` are
converted into tables with each attribute separated.
New relations are created for multi-value attributes
`rpsl_obj`'s `mnt_by` and `mbrs_by_ref`, `exchange_report`, `report_item`,
`as_set` and `route_set`'s  `num_members` and `set_members` as
`rpsl_obj_mnt_by` and `mbrs_by_ref`, `exchange_report`, `report_item`,
`as_set_contains_num`, `as_set_contains_set`, `route_set_contains_num`,
and `route_set_contains_set`.
Weak entity sets `aut_num`, `peering_set`, `filter_set`, `as_set`,
and `route_set` have their identifying entity sets' primary keys as
their primary keys, with foreign-key constraints on them.
