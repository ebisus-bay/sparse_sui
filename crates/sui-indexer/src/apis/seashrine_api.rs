use async_trait::async_trait;
use jsonrpsee::{core::RpcResult, http_client::HttpClient, RpcModule};
use jsonrpsee_proc_macros::rpc;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use sui_json_rpc::SuiRpcModule;
use sui_json_rpc_types::Page;
use sui_open_rpc::Module;
use sui_open_rpc_macros::open_rpc;
use sui_types::base_types::{ObjectID, SuiAddress};

use crate::{
    models::{
        collection::Collection,
        listing::{DisplayObject, Listing},
    },
    store::IndexerStore,
};

pub struct SeashrineApi<S> {
    fullnode: HttpClient,
    state: S,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DisplayObjectStructure {
    pub object_id: ObjectID,
    pub object_type: String,
    pub owner_address: Option<String>,
    pub name: String,
    pub link: String,
    pub description: String,
    pub project_url: String,
    pub creator: String,
    pub image_url: String,

    // Collection information.
    pub collection: CollectionStructure,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListingStructure {
    pub object_id: ObjectID,
    pub object_type: String,
    pub price: i64,
    pub lister: String,
    pub listing_status: i64,
    pub buyer: Option<String>,
    pub created_at: Option<String>,
    pub bought_at: Option<String>,

    // Listed NFT information
    pub nft: DisplayObjectStructure,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CollectionStructure {
    pub collection_object_id: ObjectID,
    pub base_type: String,
    pub mint_cap_object: Option<String>,
    pub collection_max_supply: Option<i64>,
    pub collection_current_supply: Option<i64>,
    pub bps_royalty_strategy_object: Option<String>,
    pub royalty_fee_bps: Option<i64>,
}

pub type DisplayObjectsPage = Page<DisplayObjectStructure, ObjectID>;
pub type ListingPage = Page<ListingStructure, ObjectID>;
pub type CollectionPage = Page<CollectionStructure, ObjectID>;

impl From<(Listing, (DisplayObject, Collection))> for ListingStructure {
    fn from(d: (Listing, (DisplayObject, Collection))) -> Self {
        let bought_at = match d.0.bought_at {
            Some(t) => Some(t.to_string()),
            None => None,
        };

        let created_at = match d.0.created_at {
            Some(t) => Some(t.to_string()),
            None => None,
        };
        ListingStructure {
            object_id: ObjectID::from_hex_literal(d.0.listing_id.as_str()).unwrap(),
            object_type: String::from(""),

            price: d.0.price,
            lister: d.0.lister,
            listing_status: d.0.listing_status,
            buyer: d.0.buyer,
            created_at,
            bought_at,

            nft: d.1.into(),
        }
    }
}

impl From<(DisplayObject, Collection)> for DisplayObjectStructure {
    fn from(d: (DisplayObject, Collection)) -> Self {
        DisplayObjectStructure {
            object_id: ObjectID::from_hex_literal(d.0.object_id.as_str()).unwrap(),
            object_type: d.0.object_type,
            owner_address: d.0.owner_address,
            name: d.0.name,
            link: d.0.link,
            description: d.0.description,
            project_url: d.0.project_url,
            creator: d.0.creator,
            image_url: d.0.image_url,

            collection: d.1.into(),
        }
    }
}

impl From<Collection> for CollectionStructure {
    fn from(value: Collection) -> Self {
        CollectionStructure {
            collection_object_id: ObjectID::from_hex_literal(
                value.collection_object.unwrap_or_default().as_str(),
            )
            .unwrap(),
            base_type: value.base_type,
            mint_cap_object: value.mint_cap_object,
            collection_max_supply: value.collection_max_supply,
            collection_current_supply: value.collection_current_supply,
            bps_royalty_strategy_object: value.bps_royalty_strategy_object,
            royalty_fee_bps: value.royalty_fee_bps,
        }
    }
}

impl<S: IndexerStore> SeashrineApi<S> {
    pub fn new(fullnode_client: HttpClient, state: S) -> Self {
        Self {
            fullnode: fullnode_client,
            state,
        }
    }
}

impl<S: IndexerStore> SuiRpcModule for SeashrineApi<S>
where
    S: IndexerStore + Sync + Send + 'static,
{
    fn rpc(self) -> RpcModule<Self> {
        self.into_rpc()
    }

    fn rpc_doc_module() -> Module {
        SeashrineApiStructureOpenRpc::module_doc()
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum NftFilter {
    // Filter by collection
    Collection(String),
    // Filter by owner
    Owner(SuiAddress),
    // All of them,
    All,
}

// impl Filter<

/**
 * Interface for the Seashrine API.
 */
#[open_rpc(namespace = "seashrine", tag = "Seashrine API")]
#[rpc(server, client, namespace = "seashrine")]
pub trait SeashrineApiStructure {
    #[method(name = "getDisplayObjects")]
    async fn get_display_objects(
        &self,
        query: NftFilter,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<DisplayObjectsPage>;

    #[method(name = "getActiveListings")]
    async fn get_active_listings(
        &self,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<ListingPage>;

    #[method(name = "getCollections")]
    async fn get_collections(
        &self,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<CollectionPage>;
}

/**
 * Implementation of Seashrine API interface.
 */
#[async_trait]
impl<S> SeashrineApiStructureServer for SeashrineApi<S>
where
    S: IndexerStore + Sync + Send + 'static,
{
    async fn get_display_objects(
        &self,
        query: NftFilter,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<DisplayObjectsPage> {
        self.state
            .get_display_objects_from_db(query, cursor, limit, descending_order.unwrap_or_default())
            .await
    }

    async fn get_active_listings(
        &self,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<ListingPage> {
        self.state
            .get_active_listings(cursor, limit, descending_order.unwrap_or_default())
            .await
    }

    async fn get_collections(
        &self,
        cursor: Option<ObjectID>,
        limit: Option<usize>,
        descending_order: Option<bool>,
    ) -> RpcResult<CollectionPage> {
        self.state
            .get_all_collections(cursor, limit, descending_order.unwrap_or_default())
            .await
    }
}
