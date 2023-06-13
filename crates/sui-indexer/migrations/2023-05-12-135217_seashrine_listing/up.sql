-- Your SQL goes here

/*

## Things that'll in the table.

listing_id,
price,
lister,
nft_id,
listing_status
*/

CREATE TABLE seashrine_listing (
    listing_id TEXT PRIMARY KEY,
    price BIGINT NOT NULL,
    lister address NOT NULL,
    nft_id TEXT NOT NULL,
    listing_status BIGINT NOT NULL,
    buyer address NULL,
    created_at TIMESTAMP NULL,
    bought_at TIMESTAMP NULL,

    -- Foreign key.
    FOREIGN KEY (nft_id) references display_objects(object_id)
);