mod common;

use anyhow::Result;
use common::*;
use qbittorrent_web_api_gen::QBittorrentApiGen;

#[derive(QBittorrentApiGen)]
struct Api {}

#[tokio::main]
async fn main() -> Result<()> {
    let api = Api::login(BASE_URL, USERNAME, PASSWORD).await?;

    let _ = api.search().install_plugin("https://raw.githubusercontent.com/qbittorrent/search-plugins/master/nova3/engines/legittorrents.py").await?;
    // just check that the deserialization works
    let _ = api.search().plugins().await?;

    Ok(())
}
