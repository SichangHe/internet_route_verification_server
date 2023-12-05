-- Used.
-- Query ASes, routes, reports, and report items for a given overall type
-- the reports belong to with timestamps, item count, and paging.
SELECT
    e.from_as AS source_as,
    e.to_as AS destination_as,
    e.import,
    e.overall_type,
    e.recorded_time AS exchange_report_time,
    r.observed_route_id,
    r.raw_line,
    r.address_prefix,
    r.recorded_time AS observed_route_time,
    ri.category AS report_category,
    ri.specific_case AS report_specific_case,
    ri.str_content AS report_string_content,
    ri.num_content AS report_numeric_content,
    COUNT(*) OVER () AS total_items
FROM
    exchange_report e
JOIN
    observed_route r ON e.parent_observed_route = r.observed_route_id
LEFT JOIN
    report_item ri ON e.report_id = ri.parent_report
WHERE
    e.overall_type = 'ok'
ORDER BY
    e.recorded_time
OFFSET
    0
LIMIT
    10;

-- Used.
--Query ASes, routes, reports and report items for a given specific case
--the report items belong to.
SELECT
    e.from_as AS source_as,
    e.to_as AS destination_as,
    e.import,
    e.overall_type,
    e.recorded_time AS exchange_report_time,
    r.observed_route_id,
    r.raw_line,
    r.address_prefix,
    r.recorded_time AS observed_route_time,
    ri.category AS report_category,
    ri.specific_case AS report_specific_case,
    ri.str_content AS report_string_content,
    ri.num_content AS report_numeric_content,
    COUNT(*) OVER () AS total_items
FROM
    exchange_report e
JOIN
    observed_route r ON e.parent_observed_route = r.observed_route_id
JOIN
    report_item ri ON e.report_id = ri.parent_report
WHERE
    ri.specific_case = 'err_peering'
ORDER BY
    exchange_report_time
OFFSET
    0
LIMIT
    10;

-- Used.
--Query reports and report items for a given Route object.
SELECT
    e.report_id,
    e.from_as AS source_as,
    e.to_as AS destination_as,
    e.import,
    e.overall_type,
    e.recorded_time AS exchange_report_time,
    ri.report_item_id,
    ri.category AS report_category,
    ri.specific_case AS report_specific_case,
    ri.str_content AS report_string_content,
    ri.num_content AS report_numeric_content,
    COUNT(*) OVER () AS total_items
FROM
    exchange_report e
JOIN
    report_item ri ON e.report_id = ri.parent_report
WHERE
    e.parent_observed_route = (SELECT observed_route_id FROM observed_route WHERE address_prefix = '1.0.0.0/24' LIMIT 1)
ORDER BY
    e.recorded_time
OFFSET
    0
LIMIT
    10;

-- Query verification reports for observed routes.
-- FIXME: Unrealistic. User would not have observed_route_id.
SELECT
    exchange_report.report_id,
    exchange_report.from_as,
    exchange_report.to_as,
    exchange_report.import,
    exchange_report.overall_type,
    exchange_report.parent_observed_route,
    exchange_report.recorded_time,
    observed_route.raw_line,
    observed_route.address_prefix,
    COUNT(*) OVER () AS total_items
FROM
    exchange_report
JOIN
    observed_route ON exchange_report.parent_observed_route = observed_route.observed_route_id
WHERE
    observed_route.observed_route_id = 10
ORDER BY
    exchange_report.recorded_time
OFFSET
    0
LIMIT
    10;