use rustis::{
    client::Client,
    commands::{FlushingMode, ServerCommands},
    Result,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Connect the client to a Redis server from its IP and port
    let client = Client::connect("127.0.0.1:6379").await?;

    // Flush all existing data in Redis
    client.flushdb(FlushingMode::Sync).await?;

    let value = client.acl_help().await?;
    println!("value: {value:#?}");
    assert!(value.iter().any(|e| e == "WHOAMI"));

    Ok(())
}
