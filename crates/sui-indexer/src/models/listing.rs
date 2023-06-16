use crate::models::events::Event;
use crate::schema::display_objects;
use crate::schema::seashrine_listing;
use diesel::Selectable;
use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use sui_types::base_types::SuiAddress;
use sui_types::id::{ID, UID};

/**
 *
struct CreateListingEvent has copy, store, drop {
    listing_id: ID,
    price: u64,
    lister: address,
    nft_id: ID,
    at: u64
}

struct BuyListingEvent has copy, drop, store {
    listing_id: ID,
    price: u64,
    lister: address,
    nft_id: ID,
    purchased_by: address,
    at: u64,
    royalty_fee: u64,
    marketplace_fee: u64,
}

struct UpdateListingEvent has copy, drop, store {
    listing_id: ID,
    price: u64,
    lister: address,
    nft_id: ID,
    at: u64
}
*/

#[derive(Deserialize, Debug, Clone)]
pub struct CreateListingEvent {
    listing_id: ID,
    price: u64,
    lister: SuiAddress,
    nft_id: ID,
    at: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BuyListingEvent {
    listing_id: ID,
    price: u64,
    lister: SuiAddress,
    nft_id: ID,
    purchased_by: SuiAddress,
    at: u64,
    royalty_fee: u64,
    marketplace_fee: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateListingEvent {
    listing_id: ID,
    price: u64,
    lister: SuiAddress,
    nft_id: ID,
    at: u64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ListingCreated {
    listing_id: ID,
    price: u64,
    lister: SuiAddress,
    nft_id: ID,
    // listed_at:
}

#[derive(Queryable, Selectable, Insertable, Debug, Clone)]
#[diesel(table_name = seashrine_listing)]
pub struct Listing {
    pub listing_id: String,
    pub price: i64,
    pub lister: String,
    pub nft_id: String,
    pub listing_status: i64,
    pub buyer: Option<String>,
    pub created_at: Option<chrono::NaiveDateTime>,
    pub bought_at: Option<chrono::NaiveDateTime>,
}

impl From<CreateListingEvent> for Listing {
    fn from(value: CreateListingEvent) -> Self {
        Listing {
            listing_id: value.listing_id.bytes.to_string(),
            price: value.price as i64,
            lister: value.lister.to_string(),
            nft_id: value.nft_id.bytes.to_string(),
            listing_status: 0,
            buyer: None,
            created_at: None,
            bought_at: None,
        }
    }
}

impl From<UpdateListingEvent> for Listing {
    fn from(value: UpdateListingEvent) -> Self {
        Listing {
            listing_id: value.listing_id.bytes.to_string(),
            price: value.price as i64,
            lister: value.lister.to_string(),
            nft_id: value.nft_id.bytes.to_string(),
            listing_status: 2,
            buyer: None,
            created_at: None,
            bought_at: None,
        }
    }
}

impl From<BuyListingEvent> for Listing {
    fn from(value: BuyListingEvent) -> Self {
        Listing {
            listing_id: value.listing_id.bytes.to_string(),
            price: value.price as i64,
            lister: value.lister.to_string(),
            nft_id: value.nft_id.bytes.to_string(),
            listing_status: 1,
            buyer: Some(value.purchased_by.to_string()),
            created_at: None,
            bought_at: None,
        }
    }
}

#[derive(Queryable, Insertable, Debug, Clone, Selectable)]
#[diesel(table_name = display_objects)]
#[diesel(belongs_to(Collection))]
pub struct DisplayObject {
    pub object_id: String,
    pub object_type: String,
    pub owner_address: Option<String>,
    pub name: String,
    pub link: String,
    pub description: String,
    pub project_url: String,
    pub creator: String,
    pub image_url: String,
}

/**
* struct Display<phantom T: key> has key, store {
       id: UID,
       /// Contains fields for display. Currently supported
       /// fields are: name, link, image and description.
       fields: VecMap<String, String>,
       /// Version that can only be updated manually by the Publisher.
       version: u16
   }
       struct VecMap<K: copy, V> has copy, drop, store {
        contents: vector<Entry<K, V>>,
    }
    struct Entry<K: copy, V> has copy, drop, store {
        key: K,
        value: V,
    }
*/

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Entry<K, V> {
    key: K,
    value: V,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VecMap<K, V> {
    contents: Vec<Entry<K, V>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Display {
    id: UID,
    fields: VecMap<String, String>,
    version: u16,
}

impl Display {
    /// To convert the Display into a hashmap of key values.
    pub fn into_hashmap(self) -> HashMap<String, String> {
        let vecmap_contents = self.fields.contents;
        let mut fields_map: HashMap<String, String> = HashMap::new();

        // Parse the fields
        for entry in vecmap_contents {
            fields_map.insert(entry.key, entry.value);
        }

        fields_map
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexerModuleConfig {
    pub listing_module: String,
    pub mint_cap_object: String,
    pub bps_royalty_strategy_object: String,
    pub collection_object: String,
    pub display_info: String,
    pub marker: String,
}

impl IndexerModuleConfig {
    pub fn parse() -> Self {
        let file = File::open("./indexer.config.json").unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }
}

impl Listing {
    /// Returns a set of CreateListingEvent, UpdateListingEvent, BuyListingEvent
    pub fn events_to_seashrine_listing_events(
        events: Vec<Event>,
        indexer_config: IndexerModuleConfig,
    ) -> (Vec<Listing>, Vec<Listing>, Vec<Listing>) {
        let mut create_listing_event_type: String = String::from("");
        create_listing_event_type.push_str(indexer_config.listing_module.as_str());
        create_listing_event_type.push_str("::listing::CreateListingEvent");
        let create_listing_events: Vec<Listing> = events
            .clone()
            .into_iter()
            .filter(|e| e.event_type == create_listing_event_type)
            .map(|e| {
                let parsed_event: CreateListingEvent =
                    bcs::from_bytes(e.event_bcs.as_slice()).unwrap();
                Listing {
                    listing_id: parsed_event.listing_id.bytes.to_string(),
                    price: parsed_event.price as i64,
                    lister: parsed_event.lister.to_string(),
                    nft_id: parsed_event.nft_id.bytes.to_string(),
                    listing_status: 0,
                    buyer: None,
                    created_at: Some(
                        chrono::NaiveDateTime::from_timestamp_millis(parsed_event.at as i64)
                            .unwrap(),
                    ),
                    bought_at: None,
                }
            })
            .collect();

        let mut update_listing_type: String = String::from("");
        update_listing_type.push_str(indexer_config.listing_module.as_str());
        update_listing_type.push_str("::listing::UpdateListingEvent");
        let update_listing_events: Vec<Listing> = events
            .clone()
            .into_iter()
            .filter(|e| e.event_type == update_listing_type)
            .map(|e| {
                let parsed_event: UpdateListingEvent =
                    bcs::from_bytes(e.event_bcs.as_slice()).unwrap();
                Listing {
                    listing_id: parsed_event.listing_id.bytes.to_string(),
                    price: parsed_event.price as i64,
                    lister: parsed_event.lister.to_string(),
                    nft_id: parsed_event.nft_id.bytes.to_string(),
                    listing_status: 0,
                    buyer: None,
                    created_at: Some(
                        chrono::NaiveDateTime::from_timestamp_millis(parsed_event.at as i64)
                            .unwrap(),
                    ),
                    bought_at: None,
                }
            })
            .collect();

        let mut buy_listing_type: String = String::from("");
        buy_listing_type.push_str(indexer_config.listing_module.as_str());
        buy_listing_type.push_str("::listing::BuyListingEvent");
        let buy_listing_events: Vec<Listing> = events
            .clone()
            .into_iter()
            .filter(|e| e.event_type == buy_listing_type)
            .map(|e| {
                let parsed_event: BuyListingEvent =
                    bcs::from_bytes(e.event_bcs.as_slice()).unwrap();
                Listing {
                    listing_id: parsed_event.listing_id.bytes.to_string(),
                    price: parsed_event.price as i64,
                    lister: parsed_event.lister.to_string(),
                    nft_id: parsed_event.nft_id.bytes.to_string(),
                    listing_status: 1,
                    buyer: Some(parsed_event.purchased_by.to_string()),
                    created_at: None,
                    bought_at: Some(
                        chrono::NaiveDateTime::from_timestamp_millis(parsed_event.at as i64)
                            .unwrap(),
                    ),
                }
            })
            .collect();

        (
            create_listing_events,
            update_listing_events,
            buy_listing_events,
        )
    }

    pub fn events_to_listings(
        events: Vec<Event>,
        indexer_config: IndexerModuleConfig,
    ) -> Vec<Listing> {
        let mut event_type: String = String::from("");
        event_type.push_str(indexer_config.listing_module.as_str());
        event_type.push_str("::listing::ListingCreated");

        let listing_created_events: Vec<Event> = events
            .clone()
            .into_iter()
            .filter(|e| e.event_type == event_type)
            .collect();
        let listings: Vec<Listing> = listing_created_events
            .into_iter()
            .map(|e| {
                let parsed_event: ListingCreated = bcs::from_bytes(e.event_bcs.as_slice()).unwrap();
                return Listing {
                    listing_id: parsed_event.listing_id.bytes.to_string(),
                    price: parsed_event.price as i64,
                    lister: parsed_event.lister.to_string(),
                    nft_id: parsed_event.nft_id.bytes.to_string(),
                    listing_status: 0,
                    buyer: None,
                    created_at: None,
                    bought_at: None,
                };
            })
            .collect();
        listings
    }
}
