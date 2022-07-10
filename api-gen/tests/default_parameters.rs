use anyhow::Result;
use api_gen::QBittorrentApiGen;

const USERNAME: &str = "admin";
const PASSWORD: &str = "adminadmin";
const BASE_URL: &str = "http://localhost:8080";

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
