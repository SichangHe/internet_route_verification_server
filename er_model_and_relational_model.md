# Entity-Relationship (ER) Model

An ER model representing this group project. Visualization:

Explanation and discussion for designed entity sets and relationship sets:

`rpsl_obj`, `maintainer`, and `autonomous_system` are basic entity sets
representing RPSL objects, maintainer objects, and AS objects.
The `rpsl_obj` has a multi-valued attribute `mnt_by` to store which maintainers
an RPSL object is maintained by.
`aut_num` is a weak entity set identified by its `autonomous_system` and
it is also related to `rpsl_obj` one-to-one.
Each `observed_route` has a multi-value complex attribute `exchange_report`,
and each `exchange_report` has a multi-value attribute `report_item`.
`autonomous_system` can have provider-customer relationship with each other in
the `provider_customer` relationship set,
or they can have peer-to-peer relationship with each other in
the `peer` relationship set.
`peering_set`, `filter_set`, `as_set`, and `route_set` are weak entity sets
identified by its `autonomous_system`.
`as_set` has a multi-value complex attribute `mbrs_by_ref` to refer to
its members by being referenced from the ASes,
a multi-value attribute `num_members` for specifying AS members,
and a multi-value complex attribute `set_members` for specifying AS Set members.
Same for `route_set`.

# Relational Model

Converted relational database model, visualized in a schema diagram:

Explanation for each step carried out.
