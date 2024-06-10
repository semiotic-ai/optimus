use abi::dutch_order_executor::{
    events::Fill,
    functions::{Execute, ExecuteBatch, ExecuteBatchWithCallback, ExecuteWithCallback},
};
use hex_literal::hex;
use pb::ai::semiotic::uniswap::x::{self, Orders, TransactionInfo};
use std::collections::BTreeMap;
use substreams::Hex;
use substreams_database_change::{
    pb::database::DatabaseChanges,
    tables::{PrimaryKey, Tables},
};
use substreams_ethereum::pb::sf::ethereum::r#type::v2 as eth;
use uniswap_structs::ExclusiveDutchOrder;

pub mod abi;
mod pb;
pub mod uniswap_structs;

// Uniswap X
const TRACKED_CONTRACT: [u8; 20] = hex!("6000da47483062A0D734Ba3dc7576Ce6A0B645C4");

const UNISWAP_FEE_RECIPIENT: [u8; 20] = hex!("37a8f295612602f2774d331e562be9e61B83a327");

substreams_ethereum::init!();

const EXECUTE: &[u8] = &[63u8, 98u8, 25u8, 46u8];
const EXECUTE_WITH_CALLBACK: &[u8] = &[13u8, 51u8, 88u8, 132u8];
const EXECUTE_BATCH: &[u8] = &[13u8, 122u8, 22u8, 195u8];
const EXECUTE_BATCH_WITH_CALLBACK: &[u8] = &[19u8, 251u8, 114u8, 199u8];

/// Extracts transfers events from the contract
#[substreams::handlers::map]
fn map_fills(blk: eth::Block) -> Result<x::Orders, substreams::errors::Error> {
    let block_header = blk.header.as_ref().unwrap();
    let now = block_header.timestamp.as_ref().unwrap().seconds;
    let block_number = block_header.number;
    let orders: Vec<_> = blk
        .transactions()
        .flat_map(|tx| tx.logs_with_calls())
        .filter(|(log, _call)| log.address == TRACKED_CONTRACT && Fill::match_log(log))
        .map(|(log, call)| {
            let log_index = log.index;
            let from = call.transaction.from.clone();
            let to = call.transaction.to.clone();
            let caller = call.call.caller.clone();
            let tx_hash = call.transaction.hash.clone();
            let tx_info = TransactionInfo {
                from,
                to,
                caller,
                block_time: now,
                block_number,
                log_index,
                tx_hash,
            };
            let sig = &call.call.input[0..4];
            let orders = match sig {
                EXECUTE => vec![Execute::decode(&call.call).unwrap().order],
                EXECUTE_WITH_CALLBACK => {
                    vec![ExecuteWithCallback::decode(&call.call).unwrap().order]
                }
                EXECUTE_BATCH => ExecuteBatch::decode(&call.call).unwrap().orders,
                EXECUTE_BATCH_WITH_CALLBACK => {
                    ExecuteBatchWithCallback::decode(&call.call).unwrap().orders
                }
                _ => panic!("Unknown function"),
            };
            (tx_info, orders)
        })
        .flat_map(|(tx_info, orders)| {
            orders
                .into_iter()
                .map(|(order, _signature)| ExclusiveDutchOrder::try_from(order).unwrap())
                .map(|order| order.into_proto(tx_info.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    // substreams::log::info!("New call for Fill event emmited: {:?}", maybe_data);
    Ok(Orders { orders })
}

#[substreams::handlers::map]
fn db_out(orders: Orders) -> Result<DatabaseChanges, substreams::errors::Error> {
    let mut table = Tables::new();

    for order in orders.orders {
        let tx_info = order.tx_info.unwrap();

        let mut keys = BTreeMap::new();
        keys.insert("tx_block_time".into(), tx_info.block_time.to_string());
        keys.insert("tx_block_number".into(), tx_info.block_number.to_string());
        keys.insert("tx_hash".into(), Hex(tx_info.tx_hash).to_string());
        keys.insert("tx_log_index".into(), tx_info.log_index.to_string());

        let info = order.info.unwrap();
        let input = order.input.unwrap();
        let output = order
            .outputs
            .iter()
            .filter(|o| o.recipient != UNISWAP_FEE_RECIPIENT)
            .collect::<Vec<_>>();
        let output = output.first();
        if let None = output {
            continue;
        }
        let output = (*output.unwrap()).clone();

        let decayed_fee = order
            .outputs
            .iter()
            .find(|o| o.recipient == UNISWAP_FEE_RECIPIENT)
            .map(|o| o.decayed_amount.clone())
            .unwrap_or("0".to_string());

        table
            .create_row("exclusive_dutch_auction_fills", PrimaryKey::Composite(keys))
            .set("reactor", info.reactor)
            .set("swapper", info.swapper)
            .set("nonce", info.nonce)
            .set("deadline", info.deadline)
            .set(
                "additional_validation_contract",
                info.additional_validation_contract,
            )
            .set(
                "additional_validation_data",
                info.additional_validation_data,
            )
            .set("tx_from", tx_info.from)
            .set("tx_to", tx_info.to)
            .set("tx_caller", tx_info.caller)
            .set("decay_start_time", order.decay_start_time)
            .set("decay_end_time", order.decay_end_time)
            .set("exclusive_filler", order.exclusive_filler)
            .set("exclusivity_override_bps", order.exclusivity_override_bps)
            .set("input_token", input.token)
            .set("input_start_amount", input.start_amount)
            .set("input_end_amount", input.end_amount)
            .set("input_decayed_amount", input.decayed_amount)
            .set("output_token", output.token)
            .set("output_recipient", output.recipient)
            .set("output_start_amount", output.start_amount)
            .set("output_end_amount", output.end_amount)
            .set("output_decayed_amount", output.decayed_amount)
            .set("fee_decayed_amount", decayed_fee);
    }

    Ok(table.to_database_changes())
}
