use anyhow::Result;
use api_gen::QBittorrentApiGen;

const USERNAME: &str = "admin";
const PASSWORD: &str = "adminadmin";
const BASE_URL: &str = "http://localhost:8080";

#[derive(QBittorrentApiGen)]
struct Foo {}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = Foo::login(BASE_URL, USERNAME, PASSWORD).await?;

    Ok(())
}
