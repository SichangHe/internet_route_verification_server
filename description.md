# Project Description

A REST API server for network operators to query routing policies and the verification reports for observed routes against these policies. The goal is to help operators verify the policies they publish and diagnose inconsistencies between policies and routes.

## Routing policy

Routing policies are retrieved from the Routing Policy Specification Language (RPSL) in the [Internet Route Registry (IRR)](https://www.irr.net/docs/list.html). Specified with the RPSL, each Autonomous System (AS) has its policies to accept or reject routes when importing or exporting them. Our service records these policies.

## Route verification report

We verify observed routes against the recorded policies to generate reports. Observed routes are from the [University of Oregon Route Views Archive Project](https://archive.routeviews.org/). Within each route, each import and export between two ASes is verified using their recorded policies; one report is generated for each import/export.

Each report contains both an overview and details of the verification. The overview ranges from "ok" to "bad". The details are lists of specific report items including the error types, skip reasons, and special cases.

## User task: query policies and reports

Users can query policies, routes, reports, and specific report items for a given AS, vice versa for specific use. 
For example, query for the reports related to a specific AS; and query for the ASes that have a specific type of report item.
