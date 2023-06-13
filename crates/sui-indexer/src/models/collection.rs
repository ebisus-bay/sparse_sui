use crate::schema::collections;
use diesel::{Insertable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use sui_types::{
    base_types::SuiAddress,
    id::{ID, UID},
};

use super::listing::VecMap;

/*
 * ## Types from the library.
 *
 * /// `MintCap<T>` delegates the capability of it's owner to mint `T`
 *  struct MintCap<phantom T> has key, store {
 *      /// `MintCap` ID
 *      id: UID,
 *     /// ID of the `Collection` that `MintCap` controls.
 * ///
 *     /// Intended for discovery.
 *     collection_id: ID,
 *     /// Supply that `MintCap` can mint
 *     supply: Option<Supply>,
 * }
 *
 * /// `Supply` tracks supply parameters
 * struct Supply has store, drop {
 *     max: u64,
 *     current: u64,
 * }
 *
 * /// A shared object which can be used to add receipts of type
 * /// `BpsRoyaltyStrategyRule` to `TransferRequest`.
 * struct BpsRoyaltyStrategy<phantom T> has key {
 *     id: UID,
 *     version: u64,
 *     /// Royalty charged on trades in basis points
 *     royalty_fee_bps: u16,
 *     /// Allows this middleware to touch the balance paid.
 *     /// The balance is deducted from the transfer request.
 *     /// See the docs for `BalanceAccessCap` for more info.
 *     access_cap: Option<BalanceAccessCap<T>>,
 *     /// Contains balances of various currencies.
 *     aggregator: Balances,
 *     /// If set to false, won't give receipts to `TransferRequest`.
 *     is_enabled: bool,
 * }
 * */

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Supply {
    pub max: u64,
    pub current: u64,
}

impl Supply {
    pub fn new(max: u64, current: u64) -> Self {
        Supply { max, current }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct MintCap {
    pub id: UID,
    pub collection_id: ID,
    pub supply: Option<Supply>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BalanceAccessCap {
    dummy_field: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Balances {
    inner: UID,
    items: u64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BpsRoyaltyStrategy {
    pub id: UID,
    version: u64,
    pub royalty_fee_bps: u16,
    access_cap: Option<BalanceAccessCap>,
    aggregator: Balances,
    is_enabled: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CollectionObject {
    pub id: UID,
    pub version: u64,
}

/**
*     struct Field<Name: copy + drop + store, Value: store> has key {
       /// Determined by the hash of the object ID, the field name value and it's type,
       /// i.e. hash(parent.id || name || Name)
       id: UID,
       /// The value for the name of this field
       name: Name,
       /// The value bound to this field
       value: Value,

   }

       struct Marker<phantom T> has copy, drop, store {}

   struct RoyaltyDomain has store {
       /// Royalty strategies
       strategies: VecSet<ID>,
       /// Aggregates received royalties across different coins
       aggregations: UID,
       /// Royalty share received by addresses
       royalty_shares_bps: VecMap<address, u16>,
   }

       struct VecSet<K: copy + drop> has copy, drop, store {
        contents: vector<K>,
    }
*/

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Field<Name, Value> {
    id: UID,
    name: Name,
    pub value: Value,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Marker {
    dummy_field: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct DisplayInfo {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct VecSet<K> {
    contents: Vec<K>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RoyaltyDomain {
    strategies: VecSet<ID>,
    addregations: UID,
    royalty_shares_bps: VecMap<SuiAddress, u16>,
}

#[derive(Queryable, Insertable, Debug, Clone, Selectable)]
#[diesel(table_name = collections)]
pub struct Collection {
    pub base_type: String,
    pub collection_object: Option<String>,
    pub mint_cap_object: Option<String>,
    pub collection_max_supply: Option<i64>,
    pub collection_current_supply: Option<i64>,
    pub bps_royalty_strategy_object: Option<String>,
    pub royalty_fee_bps: Option<i64>,
}

impl Collection {
    // pub fn get_base_type(object_type: String) -> String {

    // }
}
