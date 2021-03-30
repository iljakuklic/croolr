mod croolr;

use structopt::StructOpt;

/// An experimental web crawler.
///
/// Starts a server with the following API endoints:
///
/// /urls/domain.com  - List URLs discovered for given domain
///
/// /count/domain.com - Count number of discovered URLs under given domain
///
/// /crawl/domain.com - Start crawling given domain
#[derive(StructOpt,Debug)]
#[structopt(name = "croolr")]
struct Config {

    /// Port to bind to
    #[structopt(short, long, default_value="3030")]
    port: u16,

    /// IP address to listen on
    #[structopt(long, default_value="127.0.0.1")]
    host_ip: std::net::IpAddr,

    /// Max number of concurrent web requests
    #[structopt(long, name="limit", default_value="16")]
    fetch_limit: u32,
}

#[tokio::main]
async fn main() {
    let config = Config::from_args();
    croolr::server::start(config.host_ip,
                          config.port,
                          config.fetch_limit).await;
}
