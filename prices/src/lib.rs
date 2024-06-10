use std::collections::HashMap;

use pb::{semiotic::price::Token, uniswap::types::v1::Pools};
use substreams::key::segment_at;
use substreams::store::DeltaProto;
use substreams::store::{StoreNew, StoreSetIfNotExists, StoreSetIfNotExistsProto};
use substreams::{
    key::first_segment,
    pb::substreams::store_delta::Operation,
    store::DeltaExt,
    store::{DeltaBigDecimal, Deltas},
};
use substreams_database_change::pb::database::table_change;
use substreams_database_change::pb::database::DatabaseChanges;

mod pb;

const BLOCKCHAIN: &str = "ethereum";
const HOUR_IN_SECONDS: u64 = 3600;

#[substreams::handlers::map]
fn db_out(
    price_deltas: Deltas<DeltaBigDecimal>, /* store_prices */
    token_delta: Deltas<DeltaProto<Token>>,
) -> Result<DatabaseChanges, substreams::errors::Error> {
    db_out_inner(price_deltas, token_delta)
}

pub fn db_out_inner(
    price_deltas: Deltas<DeltaBigDecimal>, /* store_prices */
    token_delta: Deltas<DeltaProto<Token>>,
) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes: DatabaseChanges = Default::default();

    for delta in token_delta
        .deltas
        .iter()
        .operation_eq(Operation::Create)
    {
        let token = delta.new_value.clone();
        let mut keys = HashMap::new();
        keys.insert("blockchain".to_string(), BLOCKCHAIN.to_string());
        keys.insert("contract_address".to_string(), token.address.to_string());
        keys.insert("name".to_string(), token.name);
        keys.insert("symbol".to_string(), token.symbol);

        database_changes
            .push_change_composite(
                "tokens",
                keys,
                delta.ordinal,
                table_change::Operation::Create,
            )
            .change("decimals", (None, token.decimals));
    }

    for delta in price_deltas
        .deltas
        .iter()
        .key_first_segment_in(vec!["TokenHourData"])
        .operation_eq(Operation::Delete)
    {
        let (_, time_id, token_address) = pool_windows_id_fields(&delta.key);

        let time_id = time_id.parse::<u64>().unwrap() * HOUR_IN_SECONDS;

        let mut keys = HashMap::new();
        keys.insert("blockchain".to_string(), BLOCKCHAIN.to_string());
        keys.insert("hour".to_string(), time_id.to_string());
        keys.insert("contract_address".to_string(), token_address.to_string());

        database_changes
            .push_change_composite(
                "values",
                keys,
                delta.ordinal,
                table_change::Operation::Create,
            )
            .change("price", (None, delta.old_value.to_string()));
    }
    Ok(database_changes)
}

#[substreams::handlers::store]
fn store_token_info(pools: Pools, store: StoreSetIfNotExistsProto<Token>) {
    store_token_info_inner(pools, store)
}

pub fn store_token_info_inner(pools: Pools, store: StoreSetIfNotExistsProto<Token>) {
    for pool in pools.pools {
        let token0 = pool.token0.unwrap();
        let token1 = pool.token1.unwrap();

        store.set_if_not_exists(
            pool.log_ordinal,
            token0.address.clone(),
            &Token {
                address: token0.address,
                name: token0.name,
                symbol: token0.symbol,
                decimals: token0.decimals,
            },
        );
        store.set_if_not_exists(
            pool.log_ordinal,
            token1.address.clone(),
            &Token {
                address: token1.address,
                name: token1.name,
                symbol: token1.symbol,
                decimals: token1.decimals,
            },
        );
    }
}

pub fn pool_windows_id_fields(key: &String) -> (&str, &str, &str) {
    let table_name = first_segment(key);
    let time_id = segment_at(key, 1);
    let pool_address = segment_at(key, 2);

    return (table_name, time_id, pool_address);
}
