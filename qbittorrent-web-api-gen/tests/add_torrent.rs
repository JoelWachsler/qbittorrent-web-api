mod common;

use anyhow::Result;
use common::*;
use qbittorrent_web_api_gen::QBittorrentApiGen;

#[derive(QBittorrentApiGen)]
struct Api {}

#[tokio::main]
async fn main() -> Result<()> {
    let api = Api::login(BASE_URL, USERNAME, PASSWORD).await?;

    // assuming this torrent will exist for a while: http://www.legittorrents.info/index.php?page=torrent-details&id=5cc013e801095be61d768e609e3039da58616fd0
    const TORRENT_URL: &str = "http://www.legittorrents.info/download.php?id=5cc013e801095be61d768e609e3039da58616fd0&f=Oddepoxy%20-%20Oddepoxy%20(2013)%20[OGG%20320%20CBR].torrent";
    let _ = api.torrent_management().add(TORRENT_URL).send().await?;

    Ok(())
}
