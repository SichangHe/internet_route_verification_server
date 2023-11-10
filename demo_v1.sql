-- Enum types.
create type overall_report_type as enum (
	'ok',
	'skip',
	'unrecorded',
	'special case',
	'bad'
);
create type report_item_type as enum (
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
create table observered_route (
	observed_route_id serial primary key,
	ip_address inet not null,
	prefix_length int not null,
);
create table im_export_report (
	report_id serial primary key,
	from_as int not null,
	-- May not exist.
	to_as int,
	import bool not null,
	overall_type overall_report_type not null,
	-- TODO: Add recorded_time for the rest of the tables.
	recorded_time timestamp not null,
);
create table report_item (
	report_item_id serial primary key,
	-- TODO: Add ID for the rest of the tables without primary keys.
	category overall_report_type not null,
	specific_case report_item_type not null,
	str_content text,
	num_content int,
	parent_report int not null references im_export_report,
);
create table provide_customer(provider int not null, customer int not null);
create table aut_num(
	as_num int not null,
	as_name varchar not null
);
create table peering_set(
	--
);
	create table route_obj(
		route varchar not null,
		length int not null,
		is_v6 bool not null
	);
create table filter_set(
	--
);
	create table rpsl_obj(name varchar not null, body varchar not null);
create table peer(peer_1 int, peer_2 int);
create table as_set(
	--
);
	create table mntner(
		--
);
		create table route_set(
			--
);