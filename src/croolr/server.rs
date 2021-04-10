//! The top-level serever.

use super::crawler::Crawler;
use super::urlinfo::Domain;

use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::net::IpAddr;
use warp::Filter;

type JsonReply = Result<warp::reply::Json, warp::reject::Rejection>;

/// Start the server.
pub async fn start(ip: IpAddr, port: u16, fetch_limit: u32) {
    let crawler = Crawler::spawn(fetch_limit);

    let crawl = warp::path!("crawl" / Domain)
        .and(with_cloned(&crawler))
        .and_then(handle_crawl);

    let count = warp::path!("count" / Domain)
        .and(with_cloned(&crawler))
        .and_then(handle_count);

    let urls = warp::path!("urls" / Domain)
        .and(with_cloned(&crawler))
        .and_then(handle_urls);

    let front = warp::path::end().map(|| format!("Nothing to see here"));

    let api = front.or(crawl).or(urls).or(count);

    warp::serve(api).run((ip, port)).await;
}

/// Handle the /crawl/domain.com entry point.
async fn handle_crawl(domain: Domain, crawler: Crawler) -> JsonReply {
    let status = format!("{:?}", crawler.crawl(domain).await);
    let reply: HashMap<_, _> = [("status", &status)].iter().cloned().collect();
    Ok(warp::reply::json(&reply))
}

/// Handle the /count/domain.com entry point.
async fn handle_count(domain: Domain, crawler: Crawler) -> JsonReply {
    let num = &crawler.count_urls(domain).await;
    let reply: HashMap<_, _> = [("count", &num)].iter().cloned().collect();
    Ok(warp::reply::json(&reply))
}

/// Handle the /urls/domain.com entry point.
async fn handle_urls(domain: Domain, crawler: Crawler) -> JsonReply {
    let urls: Vec<String> = crawler
        .list_urls(domain)
        .await
        .unwrap_or(HashSet::new())
        .into_iter()
        .map(|x| x.to_string())
        .collect();
    let reply: HashMap<_, _> = [("urls", &urls)].iter().cloned().collect();
    Ok(warp::reply::json(&reply))
}

/// Warp filter to pass constant data to handlers by cloning them each time.
fn with_cloned<T: Clone + Send>(
    x: &T,
) -> impl warp::Filter<Extract = (T,), Error = Infallible> + Clone {
    let x = x.clone();
    warp::any().map(move || x.clone())
}
