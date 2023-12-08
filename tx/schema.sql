CREATE DATABASE IF NOT EXISTS ethereum;

CREATE TABLE IF NOT EXISTS ethereum.transactions (
    hash FixedString(64),
    block_number UInt64,
    block_time DateTime,
    block_hash FixedString(64),
    status FixedString(9),
    value UInt256,
    success Bool,
    gas_used UInt64,
    gas_price UInt256,
    gas_limit UInt64,
    max_fee_per_gas UInt256,
    max_priority_fee_per_gas UInt256,
    nonce UInt64,
    "index" UInt64,
    "from" FixedString(40),
    "to" FixedString(40),
    "data" String,
    "type" FixedString(20),

    PRIMARY KEY (hash, block_number, block_time, block_hash)
) Engine = MergeTree();

CREATE TABLE IF NOT EXISTS ethereum.blocks (
    time DateTime,
    "number" UInt64,
    hash FixedString(64),
    parent_hash FixedString(64),
    gas_limit UInt64,
    gas_used UInt64,
    miner FixedString(40),
    difficulty UInt256,
    total_difficulty UInt256,
    nonce UInt64,
    base_fee_per_gas UInt256,
    PRIMARY KEY (time, number, hash)
) Engine = MergeTree();

CREATE TABLE IF NOT EXISTS ethereum.cursors
(
    id         String,
	cursor     String,
	block_num  Int64,
	block_id   String
) Engine = ReplacingMergeTree() ORDER BY id;
