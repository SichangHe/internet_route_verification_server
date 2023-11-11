-- Enum types.
create type overall_report_type as enum(
	'ok',
	'skip',
	'unrecorded',
	'special case',
	'bad'
);
create type report_item_type as enum(
	"skip_regex_tilde",
	"skip_regex_with_set",
	"skip_community",
	"unrec_import_empty",
	"unrec_export_empty",
	"unrec_filter_set",
	"unrec_as_routes",
	"unrec_route_set",
	"unrec_as_set",
	"unrec_as_set_route",
	"unrec_some_as_set_route",
	"unrec_aut_num",
	"unrec_peering_set",
	"spec_uphill",
	"spec_uphill_tier1",
	"spec_tier1_pair",
	"spec_import_peer_oifps",
	"spec_import_customer_oifps",
	"spec_export_customers",
	"spec_import_from_neighbor",
	"spec_as_is_origin_but_no_route",
	"spec_as_set_contains_origin_but_no_route",
	"err_filter",
	"err_filter_as_num",
	"err_filter_as_set",
	"err_filter_prefixes",
	"err_filter_route_set",
	"err_remote_as_num",
	"err_remote_as_set",
	"err_except_peering_right",
	"err_peering",
	"err_regex",
	"rpsl_as_name",
	"rpsl_filter",
	"rpsl_regex",
	"rpsl_unknown_filter",
	"recursion",
);
-- Tables.
create table rpsl_obj(
	rpsl_obj_name text primary key,
	body text not null,
);
create table mntner(
	mntner_name text primary key references rpsl_obj,
	desc text not null,
	source text not null,
);
create table rpsl_obj_mnt_by(
	rpsl_obj_name text not null references rpsl_obj,
	mntner_name text not null references mntner,
	primary key (rpsl_obj_name, mntner_name),
);
create table aut_num(
	as_num int primary key,
	-- Nullable for AS relationship & reports.
	as_name text,
	mnt_by text references mntner,
	imports json,
	exports json,
	rpsl_obj_name text references rpsl_obj,
);
create table observed_route(
	observed_route_id serial primary key,
	raw_line text not null,
	address_prefix inet not null,
);
create table exchange_report(
	report_id serial primary key,
	from_as int not null references aut_num,
	-- May not exist.
	to_as int references aut_num,
	import bool not null,
	overall_type overall_report_type not null,
	parent_observed_route int not null references observed_route,
	-- TODO: Add recorded_time for the rest of the tables.
	recorded_time timestamp not null,
);
create table report_item(
	report_item_id serial primary key,
	category overall_report_type not null,
	specific_case report_item_type not null,
	-- May not exist.
	str_content text,
	num_content int,
	parent_report int not null references exchange_report,
);
create table provide_customer(
	provider int not null references aut_num,
	customer int not null references aut_num,
	primary key (provider, customer),
);
create table peering_set(
	peering_set_name text primary key references rpsl_obj,
	peerings json not null,
);
create table filter_set(
	filter_set_name text primary key references rpsl_obj,
	filters json not null,
);
create table route_obj(
	address_prefix inet primary key,
	origin int not null references aut_num,
	rpsl_obj_name text not null references rpsl_obj,
);
create table peer(
	peer_1 int not null references aut_num,
	peer_2 int not null references aut_num,
	primary key (peer_1, peer_2),
);
create table as_set(
	as_set_name text primary key references rpsl_obj,
	is_any boolean not null default false,
);
create table as_set_contains_num(
	as_set_name text not null references as_set,
	as_num int not null references aut_num,
	primary key (as_set_name, as_num),
);
create table as_set_contains_set(
	as_set_name text not null references as_set,
	contained_set text not null references as_set,
	primary key (as_set_name, contained_set),
);
create table mbrs_by_ref(
	rpsl_obj_name text not null references rpsl_obj,
	mntner_name text not null references mntner,
	primary key (as_set_name, mntner),
);
create table route_set(
	route_set_name text primary key references rpsl_obj,
);
create table route_set_contains_num(
	route_set_name text not null references route_set,
	as_num int not null references aut_num,
	primary key (route_set_name, as_num),
);
create table route_set_contains_set(
	route_set_name text not null references route_set,
	contained_set text not null references route_set,
	primary key (route_set_name, contained_set),
);