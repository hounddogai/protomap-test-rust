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
