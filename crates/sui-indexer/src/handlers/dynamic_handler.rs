use diesel::QueryDsl;
use futures::future::join_all;
use jsonrpsee::http_client::HttpClient;
use mysten_metrics::spawn_monitored_task;
use sui_json_rpc::api::ReadApiClient;
use sui_types::messages_checkpoint::CheckpointSequenceNumber;
use tokio::task::JoinHandle;
use tracing::log::warn;

use crate::schema::dynamic_indexing_events;
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

struct DynamicHandler {
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

        Ok(CheckpointData {
            checkpoint,
            transactions,
            changed_objects,
        })
    }

    async fn start_reindexing_for_events(&self) -> Result<(), IndexerError> {
        let last_sequence_number = self.get_last_sequence_number().await?;
        let mut current_parallel_downloads = 24;
        loop {
            // Download
            // let download_futures
        }
        Ok(())
    }
}
