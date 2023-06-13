-- Collections Table.

CREATE TABLE collections (
    base_type VARCHAR NOT NULL PRIMARY KEY,
    collection_object address NULL,
    mint_cap_object address  NULL,
    collection_max_supply BIGINT  NULL,
    collection_current_supply BIGINT  NULL,
    bps_royalty_strategy_object address NULL,
    royalty_fee_bps BIGINT NULL
);