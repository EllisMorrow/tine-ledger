# Tine Ledger — diagnostic alpha

A read-only experimental Tine plugin that validates one explicitly focused canonical Markdown ledger block. It does not create transactions, query a graph, calculate reports, or draw charts.

## Commands

- **Ledger: Show API readiness** reports the diagnostic's current capability boundary.
- **Ledger: Validate focused block (diagnostic alpha)** checks canonical CNY fields, exact decimal amounts, calendar times/UTC offsets, and basic transfer/refund field requirements. It does not verify that linked objects exist or that a graph's accounts balance.

The only declared capability is `commands.register`. The package emits notices and has no write capability, network, filesystem, or native imports. Source records remain ordinary Markdown.

## Build and verify

```text
rustup target add wasm32-unknown-unknown
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release
node scripts/test-wasm.cjs
```

Use Tine's official `plugin:check` tool before installing a package. The official Rust SDK revision and Cargo dependencies are locked.

See [Windows testing instructions](docs/TESTING.zh-CN.md). `tests/fixtures/graph/` contains synthetic records only; copy the fixtures to a separate folder before opening them in Tine.

MIT licensed. This diagnostic is not a full bookkeeping application.
