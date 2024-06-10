use anyhow::anyhow;
use ethabi::{ethereum_types::U256, Address, ParamType, Token};

use crate::pb::ai::semiotic::uniswap::x::{self, TransactionInfo};

#[derive(Debug)]
pub struct OrderInfo {
    reactor: Address,
    swapper: Address,
    nonce: U256,
    deadline: U256,
    additional_validation_contract: Address,
    additional_validation_data: Vec<u8>,
}

impl OrderInfo {
    fn params() -> Vec<ParamType> {
        vec![
            ParamType::Address,
            ParamType::Address,
            ParamType::Uint(256),
            ParamType::Uint(256),
            ParamType::Address,
            ParamType::Bytes,
        ]
    }
}

impl From<OrderInfo> for x::OrderInfo {
    fn from(value: OrderInfo) -> Self {
        //check if deadline is greater than u32
        let deadline = if value.deadline > U256::from(u32::MAX) {
            u32::MAX
        } else {
            value.deadline.as_u32()
        };
        Self {
            reactor: value.reactor.as_bytes().to_vec(),
            swapper: value.swapper.as_bytes().to_vec(),
            nonce: value.nonce.to_string(),
            deadline: deadline.to_string(),
            additional_validation_contract: value
                .additional_validation_contract
                .as_bytes()
                .to_vec(),
            additional_validation_data: value.additional_validation_data,
        }
    }
}

impl TryFrom<Vec<Token>> for OrderInfo {
    type Error = anyhow::Error;
    fn try_from(mut value: Vec<Token>) -> Result<Self, Self::Error> {
        value.reverse();
        Ok(Self {
            reactor: value.pop().unwrap().into_address().unwrap(),
            swapper: value.pop().unwrap().into_address().unwrap(),
            nonce: value.pop().unwrap().into_uint().unwrap(),
            deadline: value.pop().unwrap().into_uint().unwrap(),
            additional_validation_contract: value.pop().unwrap().into_address().unwrap(),
            additional_validation_data: value.pop().unwrap().into_bytes().unwrap(),
        })
    }
}

#[derive(Debug)]
pub struct ExclusiveDutchOrder {
    order_info: OrderInfo,
    decay_start_timer: U256,
    decay_end_timer: U256,
    exclusive_filler: Address,
    exclusivity_override_bps: U256,
    input: DutchInput,
    outputs: Vec<DutchOutput>,
}

impl ExclusiveDutchOrder {
    pub fn into_proto(self, tx_info: TransactionInfo) -> x::ExclusiveDutchOrder {
        let TransactionInfo { block_time, .. } = tx_info;
        let decay_info = DecayInformation {
            now: block_time.into(),
            decay_start_time: self.decay_start_timer.clone(),
            decay_end_time: self.decay_end_timer.clone(),
        };
        x::ExclusiveDutchOrder {
            info: Some(self.order_info.into()),
            tx_info: Some(tx_info),
            decay_start_time: self.decay_start_timer.to_string(),
            decay_end_time: self.decay_end_timer.to_string(),
            exclusive_filler: self.exclusive_filler.as_bytes().to_vec(),
            exclusivity_override_bps: self.exclusivity_override_bps.to_string(),
            input: Some(self.input.into_proto(decay_info)),
            outputs: self
                .outputs
                .into_iter()
                .map(|output| output.into_proto(decay_info))
                .collect(),
        }
    }
}

impl TryFrom<Vec<u8>> for ExclusiveDutchOrder {
    type Error = anyhow::Error;

    fn try_from(input: Vec<u8>) -> Result<Self, Self::Error> {
        let mut values = ethabi::decode(&[ParamType::Tuple(Self::params())], &input)
            .map_err(|e| anyhow!("unable to decode call.input: {:?}", e))?;
        let mut values = values.pop().unwrap().into_tuple().unwrap();
        values.reverse();
        Ok(Self {
            order_info: OrderInfo::try_from(values.pop().unwrap().into_tuple().unwrap())?,
            decay_start_timer: values.pop().unwrap().into_uint().unwrap(),
            decay_end_timer: values.pop().unwrap().into_uint().unwrap(),
            exclusive_filler: values.pop().unwrap().into_address().unwrap(),
            exclusivity_override_bps: values.pop().unwrap().into_uint().unwrap(),
            input: DutchInput::try_from(values.pop().unwrap().into_tuple().unwrap())?,
            outputs: values
                .pop()
                .unwrap()
                .into_array()
                .unwrap()
                .into_iter()
                .map(|token| DutchOutput::try_from(token.into_tuple().unwrap()).unwrap())
                .collect(),
        })
    }
}
impl ExclusiveDutchOrder {
    fn params() -> Vec<ParamType> {
        vec![
            ParamType::Tuple(OrderInfo::params()),
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Uint(256),
            ethabi::ParamType::Address,
            ethabi::ParamType::Uint(256),
            ParamType::Tuple(DutchInput::params()),
            ParamType::Array(Box::new(ParamType::Tuple(DutchOutput::params()))),
        ]
    }
}

#[derive(Debug)]
pub struct DutchInput {
    token: Address,
    start_amount: U256,
    end_amount: U256,
}

impl DutchInput {
    fn params() -> Vec<ParamType> {
        vec![
            ParamType::Address,
            ParamType::Uint(256),
            ParamType::Uint(256),
        ]
    }

    fn into_proto(self, decay_info: DecayInformation) -> x::DutchInput {
        x::DutchInput {
            token: self.token.as_bytes().to_vec(),
            start_amount: self.start_amount.to_string(),
            end_amount: self.end_amount.to_string(),
            decayed_amount: decay(decay_info, self.start_amount, self.end_amount).to_string(),
        }
    }
}

impl TryFrom<Vec<Token>> for DutchInput {
    type Error = anyhow::Error;
    fn try_from(mut value: Vec<Token>) -> Result<Self, Self::Error> {
        value.reverse();
        Ok(Self {
            token: value.pop().unwrap().into_address().unwrap(),
            start_amount: value.pop().unwrap().into_uint().unwrap(),
            end_amount: value.pop().unwrap().into_uint().unwrap(),
        })
    }
}

#[derive(Debug)]
pub struct DutchOutput {
    token: Address,
    start_amount: U256,
    end_amount: U256,
    recipient: Address,
}

impl DutchOutput {
    fn params() -> Vec<ParamType> {
        vec![
            ParamType::Address,
            ParamType::Uint(256),
            ParamType::Uint(256),
            ParamType::Address,
        ]
    }

    fn into_proto(self, decay_info: DecayInformation) -> x::DutchOutput {
        x::DutchOutput {
            token: self.token.as_bytes().to_vec(),
            start_amount: self.start_amount.to_string(),
            end_amount: self.end_amount.to_string(),
            recipient: self.recipient.as_bytes().to_vec(),
            decayed_amount: decay(decay_info, self.start_amount, self.end_amount).to_string(),
        }
    }
}

impl TryFrom<Vec<Token>> for DutchOutput {
    type Error = anyhow::Error;
    fn try_from(mut value: Vec<Token>) -> Result<Self, Self::Error> {
        value.reverse();
        Ok(Self {
            token: value.pop().unwrap().into_address().unwrap(),
            start_amount: value.pop().unwrap().into_uint().unwrap(),
            end_amount: value.pop().unwrap().into_uint().unwrap(),
            recipient: value.pop().unwrap().into_address().unwrap(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct DecayInformation {
    now: U256,
    decay_start_time: U256,
    decay_end_time: U256,
}

fn decay(
    DecayInformation {
        now,
        decay_start_time,
        decay_end_time,
    }: DecayInformation,
    start_amount: U256,
    end_amount: U256,
) -> U256 {
    if decay_end_time < decay_start_time {
        panic!("EndTimeBeforeStartTime");
    } else if decay_end_time <= now {
        end_amount
    } else if decay_start_time >= now {
        start_amount
    } else {
        let elapsed = now - decay_start_time;
        let duration = decay_end_time - decay_start_time;
        if end_amount < start_amount {
            start_amount - (start_amount - end_amount).checked_mul(elapsed).unwrap() / duration
        } else {
            start_amount + (end_amount - start_amount).checked_mul(elapsed).unwrap() / duration
        }
    }
}
