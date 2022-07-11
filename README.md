# qBittorrent web api for Rust

This is an automatic async implementation of the qBittorrent 4.1 web api. The api generation is based on the wiki markdown file which can be seen [here](https://github.com/qbittorrent/qBittorrent/wiki/WebUI-API-(qBittorrent-4.1)).

## Example

```rust
use anyhow::Result;
use qbittorrent_web_api::Api;

#[tokio::main]
async fn main() -> Result<()> {
    let api = Api::login("http://localhost:8080", "admin", "adminadmin").await?;

    // add a torrent
    api.torrent_management()
        .add("http://www.legittorrents.info/download.php?id=5cc013e801095be61d768e609e3039da58616fd0&f=Oddepoxy%20-%20Oddepoxy%20(2013)%20[OGG%20320%20CBR].torrent")
        .send()
        .await?;

    // critical logs
    let logs = api.log()
        .main()
        .critical(true)
        .warning(false)
        .normal(false)
        .info(false)
        .send()
        .await?;

    println!("{:#?}", &logs);

    // current torrent info
    let info = api.torrent_management().info().send().await?;

    println!("{:#?}", info);

    Ok(())
}

```

