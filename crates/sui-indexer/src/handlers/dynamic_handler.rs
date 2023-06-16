use diesel::QueryDsl;
use futures::future::join_all;
use jsonrpsee::http_client::HttpClient;
use mysten_metrics::spawn_monitored_task;
use sui_json_rpc::api::ReadApiClient;
use sui_types::messages_checkpoint::CheckpointSequenceNumber;
use tokio::task::JoinHandle;
use tracing::log::warn;

use crate::models::events::Event;
use crate::schema::{dynamic_indexing_events, events};
use crate::store::diesel_marco::{read_only_blocking, transactional_blocking};
use crate::store::CheckpointData;
use crate::utils::multi_get_full_transactions;
use crate::{
    errors::{Context, IndexerError},
    models::dynamic_indexing::DynamicIndexingEvent,
    PgConnectionPool,
};
use diesel::prelude::*;

use super::checkpoint_handler::{fetch_changed_objects, get_object_changes};

const MAX_EVENT_PAGE_SIZE: usize = 1000;
const PG_COMMIT_CHUNK_SIZE: usize = 1000;
const MAX_PARALLEL_DOWNLOADS: usize = 24;

pub struct DynamicHandler {
    http_client: HttpClient,
    blocking_cp: PgConnectionPool,
    events_to_index: Vec<DynamicIndexingEvent>,
    chunk_id: String,
}

impl DynamicHandler {
    pub fn new(
        http_client: HttpClient,
        events_to_index: Vec<DynamicIndexingEvent>,
        chunk_id: String,
        blocking_cp: PgConnectionPool,
    ) -> Self {
        Self {
            http_client,
            events_to_index,
            chunk_id,
            blocking_cp,
        }
    }

    pub fn spawn(self) -> Result<(), IndexerError> {
        spawn_monitored_task!(async move {
            println!("Starting reindexing for events.");
            let mut events_reindexing_response = self.start_reindexing_for_events().await;
            while let Err(e) = &events_reindexing_response {
                warn!("Issue while reindexing events.");
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                events_reindexing_response = self.start_reindexing_for_events().await;
            }
        });
        Ok(())
    }

    async fn get_last_sequence_number(&self) -> Result<i64, IndexerError> {
        read_only_blocking!(&self.blocking_cp, |conn| {
            dynamic_indexing_events::table
                .filter(dynamic_indexing_events::chunk_id.eq(Some(self.chunk_id.clone())))
                .select(diesel::dsl::max(dynamic_indexing_events::sequence_number))
                .first::<Option<i64>>(conn)
                .map(|o| o.unwrap_or(-1))
        })
        .context("Failed to get sequence number for the chunk")
    }

    async fn insert_events(&self, events: &[Event]) -> Result<(), IndexerError> {
        transactional_blocking!(&self.blocking_cp, |conn| {
            for event_chunk in events.chunks(PG_COMMIT_CHUNK_SIZE) {
                diesel::insert_into(events::table)
                    .values(event_chunk)
                    .on_conflict_do_nothing()
                    .execute(conn)
                    .map_err(IndexerError::from)
                    .context("Failed writing events to PostgresDB")?;
            }
            Ok::<(), IndexerError>(())
        })
        .context("Failed to insert events")
    }

    async fn update_sequence_number(&self, seq_no: i64) -> Result<(), IndexerError> {
        // Update the sequence number for the `chunk_id` in the `dynamic_indexing_events` table.
        transactional_blocking!(&self.blocking_cp, |conn| {
            diesel::update(dynamic_indexing_events::table)
                .filter(dynamic_indexing_events::chunk_id.eq(self.chunk_id.clone()))
                .set(dynamic_indexing_events::sequence_number.eq(seq_no))
                .execute(conn)
                .map_err(IndexerError::from)
                .context("Failed updating sequence number for chunk")?;
            Ok::<(), IndexerError>(())
        })
    }

    async fn download_checkpoint_data(
        &self,
        seq: CheckpointSequenceNumber,
    ) -> Result<CheckpointData, IndexerError> {
        let mut checkpoint = self
            .http_client
            .get_checkpoint(seq.into())
            .await
            .map_err(|e| {
                IndexerError::FullNodeReadingError(format!(
                    "Failed to get checkpoint with sequence number {} and error {:?}",
                    seq, e
                ))
            });

        while checkpoint.is_err() {
            // sleep for 0.1 second and retry if latest checkpoint is not available yet
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            // TODO(gegaowp): figure how to only measure successful checkpoint download time
            checkpoint = self
                .http_client
                .get_checkpoint(seq.into())
                .await
                .map_err(|e| {
                    IndexerError::FullNodeReadingError(format!(
                        "Failed to get checkpoint with sequence number {} and error {:?}",
                        seq, e
                    ))
                })
        }

        // unwrap here is safe because we checked for error above
        let checkpoint = checkpoint.unwrap();
        let transactions = join_all(checkpoint.transactions.chunks(50).map(|digests| {
            multi_get_full_transactions(self.http_client.clone(), digests.to_vec())
        }))
        .await
        .into_iter()
        .try_fold(vec![], |mut acc, chunk| {
            acc.extend(chunk?);
            Ok::<_, IndexerError>(acc)
        })?;

        let object_changes = transactions
            .iter()
            .flat_map(|tx| get_object_changes(&tx.effects))
            .collect::<Vec<_>>();
        let changed_objects =
            fetch_changed_objects(self.http_client.clone(), object_changes).await?;

        // println!(
        //     "Downloaded checkpoint: {:?}, {:?}",
        //     &checkpoint, &changed_objects
        // );

        Ok(CheckpointData {
            checkpoint,
            transactions,
            changed_objects,
        })
    }

    fn get_events_from_checkpointdata(checkpoint_data: &CheckpointData) -> Vec<Event> {
        let CheckpointData {
            checkpoint: _,
            transactions,
            changed_objects: _,
        } = checkpoint_data;

        transactions
            .iter()
            .flat_map(|tx| tx.events.data.iter().map(move |event| event.clone().into()))
            .collect::<Vec<_>>()
    }

    async fn start_reindexing_for_events(&self) -> Result<(), IndexerError> {
        let last_sequence_number = self.get_last_sequence_number().await?;
        let mut current_parallel_downloads = MAX_PARALLEL_DOWNLOADS;
        let mut next_cursor_sequence_number = last_sequence_number + 1;

        // loop {
        let download_futures = (next_cursor_sequence_number
            ..(next_cursor_sequence_number + current_parallel_downloads as i64))
            .map(|seq| self.download_checkpoint_data(seq as u64));
        println!("Waiting for join_all");
        let download_results = join_all(download_futures).await;

        let mut downloaded_checkpoints = vec![];
        next_cursor_sequence_number += downloaded_checkpoints.len() as i64;

        for download_result in download_results {
            if let Ok(checkpoint) = download_result {
                downloaded_checkpoints.push(checkpoint);
            } else {
                if let Err(IndexerError::UnexpectedFullnodeResponseError(fn_e)) = download_result {
                    warn!(
                        "Unexpected response from fullnode for checkpoints: {}",
                        fn_e
                    );
                } else if let Err(IndexerError::FullNodeReadingError(fn_e)) = download_result {
                    warn!("Fullnode reading error for checkpoints {}: {}. It can be transient or due to rate limiting.", next_cursor_sequence_number, fn_e);
                } else {
                    warn!("Error downloading checkpoints: {:?}", download_result);
                }
                break;
            }
        }

        current_parallel_downloads =
            std::cmp::min(downloaded_checkpoints.len() + 1, MAX_PARALLEL_DOWNLOADS);

        if downloaded_checkpoints.is_empty() {
            warn!(
                "No checkpoints were downloaded for sequence number {}, retrying...",
                next_cursor_sequence_number
            );
            // continue;
        }

        for checkpoint in downloaded_checkpoints {
            let events = Self::get_events_from_checkpointdata(&checkpoint);
            let filtered_events = events
                .into_iter()
                .filter(|event| {
                    self.events_to_index
                        .iter()
                        .any(|e| e.event_type == event.event_type)
                })
                .collect::<Vec<_>>();

            // Save the events
            self.insert_events(filtered_events.as_slice()).await?;
        }

        // Increase the indexed sequence number in the `dynamic_indexing_events` table.
        self.update_sequence_number(last_sequence_number + current_parallel_downloads as i64)
            .await?;

        Ok(())
    }
}
