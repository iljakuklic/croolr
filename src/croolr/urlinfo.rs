///! Data structures that hold information about URLs.
use std::collections::HashSet;
use std::str::FromStr;
use url::Host;

#[derive(Debug)]
pub enum CroolrError {
    Fetch(reqwest::Error),
    Status(reqwest::StatusCode),
    UnsupportedType(String),
}

pub type UrlInfo = Result<(), CroolrError>;

pub type UrlSet = HashSet<url::Url>;

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
