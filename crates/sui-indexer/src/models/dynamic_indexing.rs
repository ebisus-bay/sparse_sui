use crate::{
    get_pg_pool_connection,
    schema::{dynamic_indexing_events, dynamic_indexing_objects},
};
use diesel::prelude::*;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Insertable, PgConnection, QueryDsl, Queryable, Selectable,
};
use regex::Regex;

use super::listing::IndexerModuleConfig;

#[derive(Queryable, Insertable, Debug, Clone, Selectable)]
#[diesel(table_name = dynamic_indexing_events)]
pub struct DynamicIndexingEvent {
    pub event_type: String,
    pub sequence_number: i64,
    pub upto: i64,
    pub picked: bool,
    pub chunk_id: Option<String>,
}

#[derive(Queryable, Insertable, Debug, Clone, Selectable)]
#[diesel(table_name = dynamic_indexing_objects)]
pub struct DynamicIndexingObject {
    pub object_type: String,
    pub sequence_number: i64,
    pub upto: i64,
    pub picked: bool,
    pub chunk_id: Option<String>,
}

pub fn should_index_event_type(
    blocking_cp: &Pool<ConnectionManager<PgConnection>>,
    event_type: String,
) -> bool {
    let mut conn: diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>> =
        get_pg_pool_connection(blocking_cp).unwrap();
    let val = dynamic_indexing_events::table
        .select(dynamic_indexing_events::event_type)
        .filter(dynamic_indexing_events::event_type.eq(event_type))
        .first::<String>(&mut conn)
        .optional();

    match val {
        Ok(Some(_)) => true,
        _ => false,
    }
}

pub fn should_index_object_type(
    blocking_cp: &Pool<ConnectionManager<PgConnection>>,
    object_type: String,
) -> bool {
    let mut conn: diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>> =
        get_pg_pool_connection(blocking_cp).unwrap();
    let val = dynamic_indexing_objects::table
        .select(dynamic_indexing_objects::object_type)
        .filter(dynamic_indexing_objects::object_type.eq(object_type))
        .first::<String>(&mut conn)
        .optional();

    match val {
        Ok(Some(_)) => true,
        _ => false,
    }
}

pub fn is_display_or_collection_type(
    object_type: &str,
    indexer_config: &IndexerModuleConfig,
) -> bool {
    let display_regex = Regex::new(
        format!("0x2::display::Display<[a-zA-Z0-9_]*::[a-zA-Z0-9_]*::[a-zA-Z0-9_]*>").as_str(),
    )
    .unwrap();
    let collection_type = Regex::new(
        format!(
            "{}::collection::Collection<[a-zA-Z0-9_]*::[a-zA-Z0-9_]*::[a-zA-Z0-9_]*>",
            indexer_config.collection_object
        )
        .as_str(),
    )
    .unwrap();
    println!(
        "display, collection: {}, {}",
        display_regex.is_match(object_type),
        collection_type.is_match(object_type)
    );

    display_regex.is_match(object_type) || collection_type.is_match(object_type)
}
