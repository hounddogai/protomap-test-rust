# protomap-test-rust

A synthetic Rust code base for testing [ProtoMap](https://protomap.ai) by [HoundDog.ai](https://hounddog.ai). It is part
of the code of Acme, a fictional company.

The code mixes in, on purpose, what makes static analysis of real code bases hard, and `expected.json` is the answer key
of what a scanner should find in it.

Acme's code is spread over multiple repositories, one per programming language. They were generated together from one
catalog of Acme's APIs. Each owns its own domains and calls the others' services, so it holds copies of the protobuf
files it needs from them, byte for byte as their owners have them. Scan them together to follow calls and data from one
repository to another.

| Repository                                                                         | Language   |
|------------------------------------------------------------------------------------|------------|
| [protomap-test-csharp](https://github.com/hounddogai/protomap-test-csharp)         | C#         |
| [protomap-test-golang](https://github.com/hounddogai/protomap-test-golang)         | Go         |
| [protomap-test-java](https://github.com/hounddogai/protomap-test-java)             | Java       |
| [protomap-test-javascript](https://github.com/hounddogai/protomap-test-javascript) | JavaScript |
| [protomap-test-kotlin](https://github.com/hounddogai/protomap-test-kotlin)         | Kotlin     |
| [protomap-test-python](https://github.com/hounddogai/protomap-test-python)         | Python     |
| **protomap-test-rust** (this one)                                                  | Rust       |
| [protomap-test-swift](https://github.com/hounddogai/protomap-test-swift)           | Swift      |
| [protomap-test-typescript](https://github.com/hounddogai/protomap-test-typescript) | TypeScript |

## Repository Layout

A Cargo workspace that separates API bindings, platform infrastructure, domain crates, and server and operations
binaries. Each domain keeps its tonic adapters beside workflow commands, injected ports, and a repository that writes
audit records with SQLx. Endpoint layouts vary by domain: some implement tonic traits directly, and others use the
platform's declarative handler macro. The server binaries register every service behind an asynchronous tower identity
interceptor. The operations binaries run reconciliation, imports, and admin RPCs. Request streams are processed frame by
frame; response streams return a finite snapshot.

`proto/` holds the protobuf files, both owned and copied. `crates/proto/build.rs` uses tonic-prost-build to generate
messages and clients in Cargo's `OUT_DIR`. Generated bindings are not committed.

## Build and Run

Build and test on stable Rust, with `protoc` on `PATH` or its path in `PROTOC`. Set `ACME_TEST_DATABASE_URL` to an empty
PostgreSQL database to include the journal integration test.

```sh
cargo +stable build --workspace --locked
cargo +stable test --workspace --locked
```

To run a server, set `ACME_DATABASE_URL`, `ACME_MESH`, and `ACME_SERVICE_TOKEN`. The mesh must route each gRPC service
path to the domain that owns it. Each server has a default port, and `ACME_LISTEN` overrides the address. The operations
binaries also need `ACME_TENANT` and the workload token.

## Added Code and Benchmark Labels

`tools/grow` (seed 20261116) added ordinary application code to this repository, and 16 programs whose jobs carry rpc
responses to other rpcs through constructs a static analysis cannot resolve. All of it is under `ops/`.

- The ordinary code (domain models, validation rules, stores and caches, adjustment engines, reports, workflows, feature
  flags, retries, command-line tools, and unit tests) makes no gRPC calls, so it adds no flows.
- Each planted job reads one rpc's response and hands some of its fields to one or more implementations, each calling
  another rpc. Which implementations run is decidable from the code of the functions on the flow's path. A flow through
  one that runs is in `flows`, with a `pattern` ending in `_real`, or the plain pattern name when every implementation
  runs; one through an implementation that cannot run is in `forbidden_flows`, with the full source and target and a
  pattern ending in `_decoy`.
- `benchmark` in `expected.json` names the generator, its seed, the new patterns, the new code's path and function
  prefixes, and the planted programs' islands. The labels are complete for the new code: every real flow it creates is
  in `flows`, so a flow the scanner reports in the new code that is not there is false.
- New `flows` entries are `known_gap` unless the scanner already finds them. Unlike the other `forbidden_flows`, which
  only name a source, the `_decoy` entries name an endpoint pair; `fixture_coverage.rs`, which reads only their source
  and target kind, lists the ones the scanner reports.

| Pattern | What decides which implementation runs | True | False |
|---|---|---:|---:|
| `direct_call` | nothing: one concrete call, which the scanner finds certain | 90 | 0 |
| `dispatch_closure_map` | a map of closures, one called by a constant key | 93 | 192 |
| `dispatch_filter_loop` | a loop over keyed implementations that skips all but one key | 104 | 156 |
| `dispatch_func_value` | a function value a switch on a constant kind reassigns | 95 | 177 |
| `dispatch_index` | a list of implementations, one taken by a constant index | 79 | 136 |
| `dispatch_registry` | a registry of implementations looked up by a constant key | 119 | 226 |
| `dispatch_runtime` | run-time data, so each implementation may run | 74 | 0 |
| `dispatch_switch` | a selector's switch on a kind its caller passes as a constant | 100 | 180 |
| `fanout_all` | nothing: a loop runs every implementation | 46 | 0 |
