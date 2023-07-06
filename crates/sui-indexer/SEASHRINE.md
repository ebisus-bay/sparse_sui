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

## How to dynamically index generic objects and events.

This indexer out of the box indexes all the `0x2::display::Display<T>` objects and `<ADDRESS>::collection::Collection<T>` where `ADDRESS` is in `indexer.config.json`'s `collection_object` field. We index 0 objects and events out of the box.

### How to index generic objects/events

You need to add the type of events you want to index in the `dynamic_indexing_objects`/`dynamic_indexing_events`.

For example, if I want to index `0x3::validator::StakingRequestEvent` events I will insert this type into the `dynamic_indexing_events`.

```SQL
insert into dynamic_indexing_events
values ('0x3::validator::StakingRequestEvent', -1, -1, false, null);
```

After that it'll start indexing the all the `0x3::validator::StakingRequestEvent` events emitted from sequence number 0 to the sequence number when you inserted the query.

## RPC documentation

### seashrine_getDisplayObjects

To get all the NFTs which has a valid `Collection<T>` and `Display<T>` object.

Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "data": [
      {
        "objectId": "0x3adac051330dd4cc611210faee7560aa5a5a7af3fcd65599ee1ee16e58782e75",
        "objectType": "0xdd1ab2b964bb8aed048540282fd4f99119abb02fff8dda0b0cb3466c809e92f0::seashrine_nft::SeashrineNft",
        "ownerAddress": "0xa4c992641a50ab7d2265a5f41c2de9ca275485eea7ba225343f12fd8b0c21f5e",
        "name": "NFT #2594",
        "link": "",
        "description": "https://placekitten.com/100/100",
        "projectUrl": "Some description about the NFT.",
        "creator": "",
        "imageUrl": "",
        "collection": {
          "collectionObjectId": "0xfced30ee6d33b602212983c7dd3b784ab02e53e1b2c7cfa8d36d38a1a92184ab",
          "baseType": "0xdd1ab2b964bb8aed048540282fd4f99119abb02fff8dda0b0cb3466c809e92f0::seashrine_nft::SeashrineNft",
          "mintCapObject": "0x98d6753ccddddeea6d19d4b2dcdaf83fcd909b73239a4121733067124f7b26ef",
          "collectionMaxSupply": 0,
          "collectionCurrentSupply": 0,
          "bpsRoyaltyStrategyObject": "0x88860e39b78dffe93bd5bc24c27b5f410e680795efafbdf1b21e56a5b1f921f6",
          "royaltyFeeBps": 1000
        }
      },
      ...
    ],
    "nextCursor": null,
    "hasNextPage": false
  },
  "id": 1
}
```

### seashrine_getActiveListings

To get all the active listings that were made on seashrine sui marketplace.

Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "data": [
      {
        "objectId": "0xa45d09cf6c75fc74cac108c4c67732ddcd15f401106c9800c14a0744fd540128",
        "objectType": "",
        "price": 10000,
        "lister": "0xa4c992641a50ab7d2265a5f41c2de9ca275485eea7ba225343f12fd8b0c21f5e",
        "listingStatus": 0,
        "buyer": null,
        "createdAt": "2023-07-06 13:15:16.969",
        "boughtAt": null,
        "nft": {
          "objectId": "0x3adac051330dd4cc611210faee7560aa5a5a7af3fcd65599ee1ee16e58782e75",
          "objectType": "0xdd1ab2b964bb8aed048540282fd4f99119abb02fff8dda0b0cb3466c809e92f0::seashrine_nft::SeashrineNft",
          "ownerAddress": "0xa4c992641a50ab7d2265a5f41c2de9ca275485eea7ba225343f12fd8b0c21f5e",
          "name": "NFT #2594",
          "link": "",
          "description": "https://placekitten.com/100/100",
          "projectUrl": "Some description about the NFT.",
          "creator": "",
          "imageUrl": "",
          "collection": {
            "collectionObjectId": "0xfced30ee6d33b602212983c7dd3b784ab02e53e1b2c7cfa8d36d38a1a92184ab",
            "baseType": "0xdd1ab2b964bb8aed048540282fd4f99119abb02fff8dda0b0cb3466c809e92f0::seashrine_nft::SeashrineNft",
            "mintCapObject": "0x98d6753ccddddeea6d19d4b2dcdaf83fcd909b73239a4121733067124f7b26ef",
            "collectionMaxSupply": 0,
            "collectionCurrentSupply": 0,
            "bpsRoyaltyStrategyObject": "0x88860e39b78dffe93bd5bc24c27b5f410e680795efafbdf1b21e56a5b1f921f6",
            "royaltyFeeBps": 1000
          }
        }
      }
    ],
    "nextCursor": "0xa45d09cf6c75fc74cac108c4c67732ddcd15f401106c9800c14a0744fd540128",
    "hasNextPage": false
  },
  "id": 1
}
```

### seashrine_getCollection

To get information on all the collections from OriginByte's standard on the network.

Response

```json
{
  "jsonrpc": "2.0",
  "result": {
    "data": [
      {
        "collectionObjectId": "0xfced30ee6d33b602212983c7dd3b784ab02e53e1b2c7cfa8d36d38a1a92184ab",
        "baseType": "0xdd1ab2b964bb8aed048540282fd4f99119abb02fff8dda0b0cb3466c809e92f0::seashrine_nft::SeashrineNft",
        "mintCapObject": "0x98d6753ccddddeea6d19d4b2dcdaf83fcd909b73239a4121733067124f7b26ef",
        "collectionMaxSupply": 0,
        "collectionCurrentSupply": 0,
        "bpsRoyaltyStrategyObject": "0x88860e39b78dffe93bd5bc24c27b5f410e680795efafbdf1b21e56a5b1f921f6",
        "royaltyFeeBps": 1000
      }
    ],
    "nextCursor": "0xfced30ee6d33b602212983c7dd3b784ab02e53e1b2c7cfa8d36d38a1a92184ab",
    "hasNextPage": false
  },
  "id": 1
}
```
