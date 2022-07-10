use anyhow::Result;
use qbittorrent_web_api_gen::QBittorrentApiGen;
use tokio::time::{sleep, Duration};

const USERNAME: &str = "admin";
const PASSWORD: &str = "adminadmin";
const BASE_URL: &str = "http://localhost:8080";

#[derive(QBittorrentApiGen)]
struct Api {}

#[tokio::main]
async fn main() -> Result<()> {
    let api = Api::login(BASE_URL, USERNAME, PASSWORD).await?;

    // need a torrent in order for info to work
    const TORRENT_URL: &str = "http://www.legittorrents.info/download.php?id=5cc013e801095be61d768e609e3039da58616fd0&f=Oddepoxy%20-%20Oddepoxy%20(2013)%20[OGG%20320%20CBR].torrent";
    let _ = api.torrent_management().add(TORRENT_URL).send().await?;

    let mut tries = 5;
    while tries > 0 {
        let info = api.torrent_management().info().send().await?;
        if let Some(first) = &info.get(0) {
            // just check that something is there
            assert_ne!(first.added_on, 0);
            return Ok(());
        } else {
            tries -= 1;
            println!("torrent not found, sleeping for 1 second");
            sleep(Duration::from_secs(1)).await;
        }
    }

    panic!("Failed to find torrent!");
}
