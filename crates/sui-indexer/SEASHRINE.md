# Seashrine specific documentation

## How to run the indexer

```bash
cargo run --bin sui-indexer -- --db-url "postgresql://postgres:postgres@localhost:5432/sui_indexer" --rpc-client-url "http://127.0.0.1:9000" --rpc-server-worker --fullnode-sync-worker --rpc-server-port 8000
```

To disable warning logs from the binary use the following command:

```bash
RUST_LOG="warn=off" cargo run --bin sui-indexer -- --db-url "postgresql://postgres:postgres@localhost:5432/sui_indexer" --rpc-client-url "http://127.0.0.1:9000" --rpc-server-worker --fullnode-sync-worker --rpc-server-port 8000
```

To run the local sui node run the following command:

```bash
RUST_LOG="consensus=off" cargo run --bin sui-test-validator --jobs 1
```

## RPC documentation

### seashrine_getDisplayObjects

To get all the NFTs which has a valid `Collection<T>` and `Display<T>` object.

Response

```json

```

### seashrine_getActiveListings

To get all the active listings that were made on seashrine sui marketplace.

Response

```json

```

### seashrine_getCollection

To get information on all the collections from OriginByte's standard on the network.

Response

```json

```
