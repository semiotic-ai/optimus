use anyhow::{Ok, Result};
use substreams_ethereum::Abigen;

fn main() -> Result<(), anyhow::Error> {
    Abigen::new("DutchOrderExecutor", None,"abi/dutch-order-executor.json")?
        .generate()?
        .write_to_file("src/abi/dutch_order_executor.rs")?;

    Ok(())
}
