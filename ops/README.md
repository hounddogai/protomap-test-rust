# Operations Workspace

The platform team's tooling, built as its own Cargo workspace:

- `kit/` (`acme-kit`): text, collections, numbers, money, clocks, a TTL cache, retry policies, feature flags,
  tables, and identifiers.
- `areas/<area>/` (`acme-<area>`): the records each business area keeps, their rules, stores, engines,
  reports, and workflows, and the area's `ops_<area>_admin` tool.
- `jobs/` (`acme-ops-jobs`): the `ops_<area>_<job>` binaries that copy records between services.

```sh
cd ops && cargo build
```
