CREATE DATABASE IF NOT EXISTS prices;

CREATE TABLE IF NOT EXISTS prices.cursors
(
    id         String,
	cursor     String,
	block_num  Int64,
	block_id   String
) Engine = ReplacingMergeTree() ORDER BY id;

CREATE TABLE IF NOT EXISTS prices.tokens (
  blockchain String, 
  contract_address FixedString(40),
  name String,
  symbol String,
  decimals UInt32,

  primary key (blockchain, contract_address, name, symbol)
) Engine = MergeTree();

CREATE TABLE IF NOT EXISTS prices."values" (
  blockchain String, 
  hour DateTime,
  contract_address FixedString(40),
  price Float64,

  primary key (blockchain, hour, contract_address)
) Engine = MergeTree();
