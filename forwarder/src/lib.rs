use substreams_ethereum::pb::eth;

#[substreams::handlers::map]
fn forward(block: eth::v2::Block) -> Result<eth::v2::Block, substreams::errors::Error> {
    Ok(block)
}
