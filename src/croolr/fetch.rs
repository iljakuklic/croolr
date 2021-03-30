//! Web page fetcher.

use super::urlinfo::*;

use url::Url;
use std::future::Future;
use std::collections::HashSet;

pub type FetchResult = Result<(), CroolrError>;

/// Spawn a new task to fetch given URL.
///
/// To break inter-module dependencies, the fetcher is parametrized by two
/// callbacks. The link_cb callbakc is invoked whenever a link is encountered
/// in the page body. The finish_cb is invoked as soon as fetching finishes.
pub fn spawn<F, G>(url: Url,
                   link_cb: impl Fn(&Url) -> G + Send + Sync + 'static,
                   finish_cb: impl FnOnce(FetchResult) -> F + Send + 'static)
where F: Future + Send, F::Output: Send + 'static, G: Future + Send, G::Output: Send + 'static {
    tokio::task::spawn(async move {
        let r = do_fetch_page(url.clone(), link_cb).await;
        finish_cb(r).await
    });
}

/// Find URLs in given html document. Just quick & dirty string matching for now.
fn extract_urls<'a>(source: &'a str) -> impl Iterator<Item=String> + 'a {
    source.split("href=").filter_map(|s| {
        let q = s.chars().next()?;
        let s = s.strip_prefix(&['\"', '\''][..])?;
        let s = &s[0..(s.find(q)?)];
        escaper::decode_html(s).ok()
    })
}

/// Given base URL and a link, decide whether we should follow the link.
/// If so, return the URL to follow.
fn follow_link(base: &Url, path: &str) -> Option<Url> {
    base.join(&path).ok().filter(|l| {
        l.host() == base.host() && ["http", "https"].contains(&l.scheme())
    })
}

/// Fetch given URL and return its text if successful and all additional
/// conditions have been satisfied.
async fn fetch_url(client: &reqwest::Client, url: &Url)
        -> Result<String, CroolrError> {
    let resp = client.get(url.clone()).send().await.map_err(CroolrError::Fetch)?;

    // Check response status.
    let status = resp.status();
    if !status.is_success() {
        return Err(CroolrError::Status(status));
    }

    // Check content type is html before proceeding.
    let unsupported_type = |t: &str| CroolrError::UnsupportedType(t.to_string());
    let content_type = resp.headers()
        .get(reqwest::header::CONTENT_TYPE).ok_or(unsupported_type("unknown"))?
        .to_str().map_err(|_| unsupported_type("unparsable"))?;
    if !content_type.contains("html") {
        return Err(unsupported_type(content_type));
    }

    // Extract the page content.
    resp.text().await.map_err(CroolrError::Fetch)
}

/// Fetch given page and extract URLs, calling link_cb on each.
async fn do_fetch_page<F>(url: Url, link_cb: impl Fn(&Url) -> F) -> FetchResult
where F: Future + Send, F::Output: Send + 'static {
    let client = reqwest::Client::new();
    let body = fetch_url(&client, &url).await?;
    let mut duplicates = HashSet::new();
    for link in extract_urls(&body).filter_map(|l| follow_link(&url, &l)) {
        if duplicates.contains(&link) {
            continue;
        }
        link_cb(&link).await;
        duplicates.insert(link);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    // Check the decision whether particular link should be followed.
    #[test]
    fn unit_follow_link() {
        let base = Url::parse("http://example.com/xyz/").unwrap();
        assert_eq!(follow_link(&base, "/foo"),
                   Url::parse("http://example.com/foo").ok());
        assert_eq!(follow_link(&base, "foo"),
                   Url::parse("http://example.com/xyz/foo").ok());
        assert_eq!(follow_link(&base, "http://example.com/here"),
                   Url::parse("http://example.com/here").ok());
        assert!(follow_link(&base, "http://nothing.io").is_none());
        assert!(follow_link(&base, "ftp://example.com/here").is_none());
    }

    // A number of absolute and relative URLs (and other strings) for testing.
    const TEST_URLS: &[&str] = &[
        "foo.png",
        "http://bar.xyz",
        "random.html",
        "..",
        ".",
        "?x=y&a=b",
        "'\">>>",
    ];

    // Check HTML link parsing.

    #[test]
    fn unit_parse_link_ok() {
        for url in TEST_URLS {
            let html = format!("<a href=\"{}\">", escaper::encode_attribute(url));
            let mut it = extract_urls(&html);
            assert_eq!(it.next(), Some(url.to_string()), "URL parsing failed");
            assert_eq!(it.next(), None, "URL parser returns too many items");
        }
    }

    #[test]
    fn unit_parse_link_many() {
        let mut html = String::new();
        for url in TEST_URLS {
            html.push_str(&format!("  <li><a href='{}'>Link</a></li>\n",
                                   escaper::encode_attribute(url)));
        }
        let html = format!("<ul>\n{}</ul>\n", html);
        assert!(extract_urls(&html).eq(TEST_URLS.iter().map(|u| *u)),
                "Parser extracts incorrect URLs");
    }

    #[test]
    fn unit_parse_link_bad_html() {
        let test_cases = &[
            "<a href=\"earlyend",
            "<a href=missingquotes",
        ];
        for html in test_cases {
            assert!(extract_urls(&html).next().is_none(), "Parser matches on garbage");
        }
    }

    #[test] #[ignore]
    fn unit_parse_link_bad_html_failing() {
        // These cases not yet supported by the hacky parser.
        let test_cases = &[
            "<ahref=\"/\"",
            "<a attr_ending_in_href=\"/\"",
            "not in tag href=\"/\"",
        ];
        for html in test_cases {
            assert!(extract_urls(&html).next().is_none(), "Parser matches on garbage");
        }
    }

}
