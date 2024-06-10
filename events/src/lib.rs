mod abi;
mod db;
mod events;

use std::collections::HashMap;

use abi::Events;
use events::ToTableChange;
use prost_types::Timestamp;
use substreams_database_change::pb::database::DatabaseChanges;

use substreams::Hex;
use substreams_ethereum::pb::eth;

use crate::db::push_create;

pub struct EvtTxInfo {
    contract_address: Vec<u8>,
    evt_tx_hash: Vec<u8>,
    tx_from: Vec<u8>,
    tx_to: Vec<u8>,
    evt_block_number: u64,
    evt_block_time: Timestamp,
    evt_index: u32,
}

pub struct EventWithInfo<T> {
    pub event: T,
    pub info: EvtTxInfo,
}

substreams_ethereum::init!();

fn get_events(
    block: &eth::v2::Block,
) -> Result<Vec<EventWithInfo<Events>>, substreams::errors::Error> {
    let number = block.number;
    let header = block.header.as_ref().unwrap();
    
    let events = block
        .logs()
        .flat_map(|view| {
            
            Events::match_and_decode(view.log).map(|event| EventWithInfo {
                info: EvtTxInfo {
                    contract_address: view.log.address.clone(),
                    evt_tx_hash: view.receipt.transaction.hash.clone(),
                    tx_to: view.receipt.transaction.to.clone(),
                    tx_from: view.receipt.transaction.from.clone(),
                    evt_block_number: number,
                    evt_block_time: header.timestamp.clone().unwrap(),
                    evt_index: view.log.index,
                },
                event,
            })
        })
        .collect();

    Ok(events)
}

#[substreams::handlers::map]
fn db_out(block: eth::v2::Block) -> Result<DatabaseChanges, substreams::errors::Error> {
  transform(block)
}

pub fn transform(block: eth::v2::Block) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes: DatabaseChanges = Default::default();
    let events = get_events(&block)?;

    transform_events_to_database_changes(&mut database_changes, events);

    Ok(database_changes)
}



fn transform_events_to_database_changes(
    changes: &mut DatabaseChanges,
    events: Vec<EventWithInfo<Events>>,
) {
    for evt in events {
        let table_name = evt.event.get_table_name();
        let mut keys: HashMap<String, String> = HashMap::new();
        keys.insert(
            "evt_tx_hash".to_string(),
            Hex(evt.info.evt_tx_hash.clone()).to_string(),
        );
        keys.insert("evt_index".to_string(), evt.info.evt_index.to_string());
        keys.insert(
            "evt_block_number".to_string(),
            evt.info.evt_block_number.to_string(),
        );
        keys.insert(
            "evt_block_time".to_string(),
            evt.info.evt_block_time.to_string(),
        );
        push_create(changes, table_name, keys, 0, evt);
    }
}
