-- create demo database version
create table im_export_report (
	import bool not null,
	overall_type enum not null
);

create table report_item(
	category enum not null,
	specific_case enum not null,
	str_content varchar(50),
	num_content int
	
);
create table path(
	--
);
create table observered_route(
	--	
);
create table provide_customer(
	provider int not null,
	customer int not null
);
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
create table rpsl_obj(
	name varchar not null,
	body varchar not null
);
create table peer(
	peer_1 int,
	peer_2 int
);
create table as_set(
	--
);
create table mntner(
	--
);
create table route_set(
	--
);
