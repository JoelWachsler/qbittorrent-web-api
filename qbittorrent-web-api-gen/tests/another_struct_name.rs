mod common;

use anyhow::Result;
use common::*;
use qbittorrent_web_api_gen::QBittorrentApiGen;

#[derive(QBittorrentApiGen)]
struct Foo {}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = Foo::login(BASE_URL, USERNAME, PASSWORD).await?;

    Ok(())
}
