// @generated
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Orders {
    #[prost(message, repeated, tag="1")]
    pub orders: ::prost::alloc::vec::Vec<ExclusiveDutchOrder>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionInfo {
    #[prost(bytes="vec", tag="1")]
    pub from: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="2")]
    pub to: ::prost::alloc::vec::Vec<u8>,
    /// maker?
    #[prost(bytes="vec", tag="3")]
    pub caller: ::prost::alloc::vec::Vec<u8>,
    #[prost(int64, tag="4")]
    pub block_time: i64,
    #[prost(uint64, tag="5")]
    pub block_number: u64,
    #[prost(bytes="vec", tag="6")]
    pub tx_hash: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag="7")]
    pub log_index: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OrderInfo {
    #[prost(bytes="vec", tag="1")]
    pub reactor: ::prost::alloc::vec::Vec<u8>,
    /// taker
    #[prost(bytes="vec", tag="2")]
    pub swapper: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="3")]
    pub nonce: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub deadline: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="5")]
    pub additional_validation_contract: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes="vec", tag="6")]
    pub additional_validation_data: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExclusiveDutchOrder {
    #[prost(message, optional, tag="1")]
    pub info: ::core::option::Option<OrderInfo>,
    #[prost(message, optional, tag="2")]
    pub tx_info: ::core::option::Option<TransactionInfo>,
    #[prost(string, tag="3")]
    pub decay_start_time: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub decay_end_time: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="5")]
    pub exclusive_filler: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="6")]
    pub exclusivity_override_bps: ::prost::alloc::string::String,
    #[prost(message, optional, tag="7")]
    pub input: ::core::option::Option<DutchInput>,
    #[prost(message, repeated, tag="8")]
    pub outputs: ::prost::alloc::vec::Vec<DutchOutput>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DutchOutput {
    /// token out
    #[prost(bytes="vec", tag="1")]
    pub token: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="2")]
    pub start_amount: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub end_amount: ::prost::alloc::string::String,
    /// target?
    #[prost(bytes="vec", tag="4")]
    pub recipient: ::prost::alloc::vec::Vec<u8>,
    /// token out raw: calculate using decay()
    #[prost(string, tag="5")]
    pub decayed_amount: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DutchInput {
    /// token in
    #[prost(bytes="vec", tag="1")]
    pub token: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="2")]
    pub start_amount: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub end_amount: ::prost::alloc::string::String,
    /// token in raw: calculate using decay()
    #[prost(string, tag="4")]
    pub decayed_amount: ::prost::alloc::string::String,
}
// @@protoc_insertion_point(module)
