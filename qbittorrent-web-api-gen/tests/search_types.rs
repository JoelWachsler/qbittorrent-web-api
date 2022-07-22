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
    let plugins = api.search().plugins().await?;
    eprintln!("{:?}", plugins);
    // let _ = api.search().results(1).send().await?;
    // let _ = api.search().delete(1).await?;

    Ok(())
}
