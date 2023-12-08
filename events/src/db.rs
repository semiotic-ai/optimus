use std::collections::HashMap;

use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};

use crate::{
    events::{TableField, ToTableChange},
    EventWithInfo, abi::Events,
};

pub fn push_create(
    changes: &mut DatabaseChanges,
    table_name: &str,
    keys: HashMap<String, String>,
    ordinal: u64,
    value: EventWithInfo<Events>,
)
{
    let table_change = changes.push_change_composite(
        table_name,
        keys,
        ordinal,
        Operation::Create,
    );

    // default event tables
    table_change
        .change("evt_index", (None, value.info.evt_index))
        .change("evt_tx_hash", (None, &value.info.evt_tx_hash.get_value()))
        .change("tx_from", (None, &value.info.tx_from.get_value()))
        .change("tx_to", (None, &value.info.tx_to.get_value()))
        .change("evt_block_time", (None, value.info.evt_block_time))
        .change("evt_block_number", (None, value.info.evt_block_number))
        .change(
            "contract_address",
            (None, &value.info.contract_address.get_value()),
        );

    // event specific
    value.event.add_table_changes(table_change);
}
