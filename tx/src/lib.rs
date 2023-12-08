use std::collections::HashMap;

use substreams::{scalar::BigInt, Hex};
use substreams_database_change::pb::database::{table_change::Operation, DatabaseChanges};
use substreams_ethereum::pb::eth::{
    self,
    v2::{balance_change::Reason, transaction_trace, TransactionTrace, TransactionTraceStatus},
};

substreams_ethereum::init!();

#[substreams::handlers::map]
fn db_out(block: eth::v2::Block) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut database_changes: DatabaseChanges = Default::default();

    create_block_change(&mut database_changes, &block);

    block.transaction_traces.iter().for_each(|tx| {
        create_transaction_change(&mut database_changes, tx, &block);
    });

    Ok(database_changes)
}

fn create_transaction_change(
    database_changes: &mut DatabaseChanges,
    tx: &TransactionTrace,
    block: &eth::v2::Block,
) {
    let block_number = block.number;
    let block_time = block.timestamp().clone();
    let block_hash = block.hash.clone();
    // let base_fee = block.header.unwrap().base_fee_per_gas;
    let mut keys = HashMap::new();
    keys.insert("hash".into(), hex_string(tx.hash.clone()));
    keys.insert("block_number".into(), block_number.to_string());
    keys.insert("block_time".into(), block_time.to_string());
    keys.insert("block_hash".into(), hex_string(block_hash));
    let change = database_changes.push_change_composite(
        "transactions",
        keys,
        tx.begin_ordinal,
        Operation::Create,
    );
    let status = TransactionTraceStatus::from_i32(tx.status).unwrap();
    change.change("status", (None, status.as_str_name()));
    change.change("value", (None, big_int_to_string(tx.value.clone())));
    change.change(
        "success",
        (None, status == TransactionTraceStatus::Succeeded),
    );
    change.change("gas_used", (None, tx.gas_used));
    change.change("gas_limit", (None, tx.gas_limit));
    change.change("gas_price", (None, big_int_to_string(tx.gas_price.clone())));
    change.change(
        "max_fee_per_gas",
        (None, big_int_to_string(tx.max_fee_per_gas.clone())),
    );
    change.change(
        "max_priority_fee_per_gas",
        (None, big_int_to_string(tx.max_priority_fee_per_gas.clone())),
    );
    change.change("nonce", (None, tx.nonce));
    change.change("index", (None, tx.index));
    change.change("from", (None, hex_string(tx.from.clone())));
    change.change("to", (None, hex_string(tx.to.clone())));
    change.change("data", (None, hex_string(tx.return_data.clone())));
    let r#type = transaction_trace::Type::from_i32(tx.r#type).unwrap();
    change.change("type", (None, r#type.as_str_name()));
    // change.change("access_list", (None, tx.access_list.trace_address.clone().join(",")));
    // change.change("priority_fee_per_gas", (None, ))
}

fn create_block_change(database_changes: &mut DatabaseChanges, block: &eth::v2::Block) {
    let block_number = block.number;
    let header = block.header.as_ref();
    let block_time = block.timestamp();
    if let Some(header) = header {
        let header = header.clone();
        let mut block_keys = HashMap::new();
        block_keys.insert("time".into(), block_time.clone().to_string());
        block_keys.insert("number".into(), block_number.to_string());
        block_keys.insert("hash".into(), Hex(block.hash.clone()).to_string());
        let block_change =
            database_changes.push_change_composite("blocks", block_keys, 0, Operation::Create);
        block_change.change("parent_hash", (None, hex_string(header.parent_hash)));
        block_change.change("gas_limit", (None, header.gas_limit));
        block_change.change("gas_used", (None, header.gas_used));
        let miner = block
            .balance_changes
            .iter()
            .find(|bc| bc.reason == Reason::RewardMineBlock as i32)
            .map_or(
                "0000000000000000000000000000000000000000".to_string(),
                |m| hex_string(m.address.clone()),
            );

        block_change.change("miner", (None, miner));
        block_change.change(
            "difficulty",
            (
                None,
                header.difficulty.map_or("0".to_string(), |d| {
                    BigInt::from_unsigned_bytes_be(d.bytes.as_slice()).to_string()
                }),
            ),
        );
        block_change.change(
            "total_difficulty",
            (None, big_int_to_string(header.total_difficulty)),
        );
        block_change.change("nonce", (None, header.nonce));
        block_change.change(
            "base_fee_per_gas",
            (None, big_int_to_string(header.base_fee_per_gas)),
        );
    }
}

fn big_int_to_string(big_int: Option<eth::v2::BigInt>) -> String {
    big_int.map_or("0".to_string(), |f| {
        BigInt::from_unsigned_bytes_be(f.bytes.as_slice()).to_string()
    })
}

fn hex_string(hash: Vec<u8>) -> String {
    Hex(hash).to_string()
}
