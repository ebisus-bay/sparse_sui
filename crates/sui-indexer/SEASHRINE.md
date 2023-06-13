# Seashrine specific documentation

## How to run the indexer

```bash
cargo run --bin sui-indexer -- --db-url "postgresql://postgres:postgres@localhost:5432/sui_indexer" --rpc-client-url "http://127.0.0.1:9000" --rpc-server-worker --fullnode-sync-worker --rpc-server-port 8000
```

## RPC documentation
