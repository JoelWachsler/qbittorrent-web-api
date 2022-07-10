use anyhow::Result;
use qbittorrent_web_api_gen::QBittorrentApiGen;

const USERNAME: &str = "admin";
const PASSWORD: &str = "adminadmin";
const BASE_URL: &str = "http://localhost:8080";

#[derive(QBittorrentApiGen)]
struct Api {}

#[tokio::main]
async fn main() -> Result<()> {
    let api = Api::login(BASE_URL, USERNAME, PASSWORD).await?;
    api.logout().await?;

    Ok(())
}