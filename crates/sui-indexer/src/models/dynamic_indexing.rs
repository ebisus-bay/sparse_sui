use crate::{get_pg_pool_connection, schema::dynamic_indexing_events};
use diesel::prelude::*;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    Insertable, PgConnection, QueryDsl, Queryable, Selectable,
};

#[derive(Queryable, Insertable, Debug, Clone, Selectable)]
#[diesel(table_name = dynamic_indexing_events)]
pub struct DynamicIndexingEvent {
    pub event_type: String,
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
