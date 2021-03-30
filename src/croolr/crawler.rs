//! The main crawler module.

use super::urlinfo::*;
use super::fetch;

use url::Url;
use tokio::sync::{oneshot, mpsc};
use std::collections::{HashSet, HashMap};

/// A handle to the crawler process. Used to send messages to it.
#[derive(Clone)]
pub struct Crawler {
    channel: mpsc::Sender<Message>,
}

/// Reply to a request to crawl given domain.
#[derive(Debug)]
pub enum CrawlReply {
    /// Request has been queued.
    Queued,
    /// THe requested host name is malformed.
    MalformedHostName(url::ParseError),
    /// This domain has already been crawled or is crawling.
    AlreadyCrawling,
}

/// Reply to URL listing.
pub type ListUrlsReply = Option<UrlSet>;

/// Reply to URL count request.
pub type CountUrlsReply = Option<usize>;

/// Messages the main crawler is capable reacting to.
#[derive(Debug)]
enum Message {
    /// Notify that a (possibly) new URL has been found in a web page code.
    LinkFound(Url),
    /// Notify that a web page has been processed with given result.
    Processed(Url, UrlInfo),
    /// Crawl given domain.
    Crawl(String, oneshot::Sender<CrawlReply>),
    /// Get urls for given domain.
    ListUrls(String, oneshot::Sender<ListUrlsReply>),
    /// Get the number of urls for given domain.
    CountUrls(String, oneshot::Sender<CountUrlsReply>),
}

// Crawler agent implementation.
impl Crawler {

    /// Start a new crawler.
    ///
    /// The fetch_limit argument specifies max number of concurrent http downloads.
    /// Returns a handle that can be used to communicate with the crawler.
    /// Panics if fetch_limit is 0.
    pub fn spawn(fetch_limit: u32) -> Crawler {
        assert!(fetch_limit >= 1, "Fetch limit must be at least 1");
        let (sx, rx) = mpsc::channel(32);
        let crawler = Crawler {channel: sx};
        tokio::task::spawn(crawler.clone().run(rx, fetch_limit));
        crawler
    }

    /// Instruct the crawler to crawl given domain.
    pub async fn crawl(&self, domain: String) -> CrawlReply {
        self.send_and_wait_reply(|r| Message::Crawl(domain, r)).await
    }

    /// Instruct the crawler to send a list of URLs for given domain.
    pub async fn list_urls(&self, domain: String) -> ListUrlsReply {
        self.send_and_wait_reply(|r| Message::ListUrls(domain, r)).await
    }

    /// Instruct the crawler to send a list of URLs for given domain.
    pub async fn count_urls(&self, domain: String) -> CountUrlsReply {
        self.send_and_wait_reply(|r| Message::CountUrls(domain, r)).await
    }

    /// Main crawler message handling loop.
    async fn run(self, mut rx: mpsc::Receiver<Message>, mut fetch_limit: u32) {
        let mut seen: HashSet<Url> = HashSet::new();
        let mut data: HashMap<String, UrlSet> = HashMap::new();
        let mut fetch_queue = Vec::new();

        while let Some(msg) = rx.recv().await {
            match msg {
            Message::LinkFound(url) => {
                if !seen.contains(&url) {
                    seen.insert(url.clone());
                    if fetch_limit > 0 {
                        fetch_limit -= 1;
                        self.fetch(url);
                    } else {
                        fetch_queue.push(url);
                    }
                }
            },
            Message::Processed(url, _info) => {
                if let Some(host) = url.host_str() {
                    let domain_data = data.entry(host.to_string()).or_default();
                    domain_data.insert(url);
                }
                match fetch_queue.pop() {
                    Some(next_url) => self.fetch(next_url),
                    None => fetch_limit += 1,
                }
            },
            Message::ListUrls(host, reply) => {
                reply.send(data.get(&host).cloned()).unwrap();
            },
            Message::CountUrls(host, reply) => {
                reply.send(data.get(&host).map(|x| x.len())).unwrap();
            },
            Message::Crawl(host, reply) => {
                let ret = match url_from_host(&host) {
                Ok(url) => {
                    if seen.contains(&url) {
                        CrawlReply::AlreadyCrawling
                    } else {
                        self.send(Message::LinkFound(url)).await;
                        CrawlReply::Queued
                    }
                },
                Err(err) => CrawlReply::MalformedHostName(err)
                };
                let _ = reply.send(ret);
            },
            }
        }
    }

    /// Fetch given page
    fn fetch(&self, url: Url) {
        let h_link = self.clone();
        let cb_link = move |u: &Url| {
            let h = h_link.clone();
            let u = u.clone();
            async move {
                h.send(Message::LinkFound(u)).await;
            }
        };

        let h_finish = self.clone();
        let url2 = url.clone();
        let cb_finish = |r| async move {
            h_finish.send(Message::Processed(url2, r)).await;
        };

        fetch::spawn(url, cb_link, cb_finish);
    }

    /// Send a message to the crawler.
    async fn send(&self, msg: Message) {
        self.channel.send(msg).await.unwrap()
    }

    /// Send a message to the crawler and wait for reply.
    async fn send_and_wait_reply<F, R>(&self, msg_func: F) -> R 
    where F: FnOnce(oneshot::Sender<R>) -> Message {
        let (sx, rx) = oneshot::channel();
        self.send(msg_func(sx)).await;
        rx.await.unwrap()
    }

}

fn url_from_host(host: &str) -> Result<Url, url::ParseError> {
    let mut u = Url::parse("http://localhost").unwrap();
    u.set_host(Some(host))?;
    Ok(u)
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn test_url_discovered() {
        let crawler = Crawler::spawn(8);
        let url = Url::parse("http://example.com/foo.png").unwrap();
        crawler.send(Message::Processed(url.clone(), Ok(()))).await;
        let ret = crawler.list_urls("example.com".to_string()).await
            .expect("domain not present");
        assert!(ret.len() == 1, "Too many URLs present");
        assert!(ret.contains(&url));
    }

}
