///! Data structures that hold information about URLs.

use std::collections::HashSet;

#[derive(Debug)]
pub enum CroolrError {
    Fetch(reqwest::Error),
    Status(reqwest::StatusCode),
    UnsupportedType(String),
}

pub type UrlInfo = Result<(), CroolrError>;

pub type UrlSet = HashSet<url::Url>;
