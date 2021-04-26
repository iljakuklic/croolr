use serde::ser::Serialize;
///! Data structures that hold information about URLs.
use std::collections::HashMap;
use std::str::FromStr;
use url::Host;

#[derive(Debug, Clone)]
pub enum Error {
    Fetch(String),
    Status(reqwest::StatusCode),
    UnsupportedType(String),
}

pub type FetchResult = Result<reqwest::StatusCode, Error>;

/// Stores metadata about an URL.
#[derive(Debug, Clone)]
pub struct UrlInfo(pub FetchResult);

impl Serialize for UrlInfo {
    fn serialize<S>(&self, s: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match &self.0 {
            Ok(status) => s.serialize_newtype_variant("urlinfo", 0, "ok", &status.to_string()),
            Err(Error::Fetch(e)) => {
                s.serialize_newtype_variant("urlinfo", 1, "fetch_error", e)
            }
            Err(Error::Status(e)) => {
                s.serialize_newtype_variant("urlinfo", 2, "response_error", &e.to_string())
            }
            Err(Error::UnsupportedType(e)) => {
                s.serialize_newtype_variant("urlinfo", 3, "unsupported_mime", e)
            }
        }
    }
}

pub type UrlSet = HashMap<url::Url, UrlInfo>;

/// Domain name, enforced to be lower case.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Domain(String);

impl Domain {
    pub fn from_host<S: AsRef<str>>(h: &Host<S>) -> Self {
        Domain(h.to_string())
    }
}

impl FromStr for Domain {
    type Err = url::ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        url::Host::parse(s).map(|h| Self::from_host(&h))
    }
}

impl std::ops::Deref for Domain {
    type Target = str;
    fn deref(&self) -> &str {
        self.0.deref()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn unit_domain_case_insensitive() {
        assert!(Domain::from_str("eXamPle.coM") == "ExamPle.Com".parse())
    }
}
