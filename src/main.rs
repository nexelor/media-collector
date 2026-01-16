use anyhow::Result;

mod database;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    Ok(())
}
