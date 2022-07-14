mod common;

use anyhow::Result;
use common::*;
use qbittorrent_web_api_gen::QBittorrentApiGen;

#[derive(QBittorrentApiGen)]
struct Api {}

#[tokio::main]
async fn main() -> Result<()> {
    let api = Api::login(BASE_URL, USERNAME, PASSWORD).await?;

    let _ = api
        .log()
        .main()
        .normal(true)
        .info(false)
        .warning(true)
        .critical(false)
        .send()
        .await?;

    Ok(())
}
