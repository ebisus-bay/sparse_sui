-- Your SQL goes here

-- Table to store which events to index.

create table dynamic_indexing_events (
    event_type text primary key,
    -- sequence_number tracks the indexing from 0 -> upto
    sequence_number bigint not null,
    -- upto is the last sequence number that needs to be index
    upto bigint not null,
    -- to track whether a thread is already indexing this event type
    picked boolean not null default false,
    chunk_id text null
);
