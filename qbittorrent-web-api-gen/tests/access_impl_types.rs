use anyhow::Result;

mod foo {
    use qbittorrent_web_api_gen::QBittorrentApiGen;

    #[allow(dead_code)]
    #[derive(QBittorrentApiGen)]
    struct Api {}
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ = foo::api_impl::ApplicationPreferencesBittorrentProtocol::TCP;

    Ok(())
}
