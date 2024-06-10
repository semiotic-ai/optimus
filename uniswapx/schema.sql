CREATE DATABASE IF NOT EXISTS uniswap_x_ethereum;

CREATE TABLE IF NOT EXISTS uniswap_x_ethereum.cursors
(
  id         String,
	cursor     String,
	block_num  Int64,
	block_id   String
) Engine = ReplacingMergeTree() ORDER BY id;

CREATE TABLE IF NOT EXISTS uniswap_x_ethereum.exclusive_dutch_auction_fills
(
    reactor FixedString(40),
    swapper FixedString(40),
    nonce UInt256,
    deadline DateTime,
    additional_validation_contract FixedString(40),
    additional_validation_data String,
    
    tx_from FixedString(40),
    tx_to FixedString(40),
    tx_caller FixedString(40),
    tx_block_time DateTime,
    tx_block_number UInt64,
    tx_hash FixedString(64),
    tx_log_index UInt32,
    
    decay_start_time DateTime,
    decay_end_time DateTime,
    exclusive_filler FixedString(40),
    exclusivity_override_bps UInt256,
    
    input_token FixedString(40),
    input_start_amount UInt256,
    input_end_amount UInt256,
    input_decayed_amount UInt256,
    
    output_token FixedString(40),
    output_recipient FixedString(40),
    output_start_amount UInt256,
    output_end_amount UInt256,
    output_decayed_amount UInt256,

    fee_decayed_amount UInt256,
) ENGINE = MergeTree()
PARTITION BY toYYYYMM(toDateTime(tx_block_time))
ORDER BY (tx_block_time, tx_block_number, tx_hash, tx_log_index);
