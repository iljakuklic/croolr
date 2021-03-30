///! Data structures that hold information about URLs.

use std::collections::HashSet;
use std::str::FromStr;

#[derive(Debug)]
pub enum CroolrError {
    Fetch(reqwest::Error),
    Status(reqwest::StatusCode),
    UnsupportedType(String),
}

pub type UrlInfo = Result<(), CroolrError>;

pub type UrlSet = HashSet<url::Url>;

/// Domain name, enforced to be lower case.
#[derive(Debug,Clone,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct Domain(String);

impl Domain {
    /// Construct a new domain, forcing it to be lowercase.
    pub fn new(d: &str) -> Self {
        Domain(d.to_lowercase())
    }
}

impl FromStr for Domain {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Domain::new(s))
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
        assert!(Domain::new("eXamPle.coM") == Domain::new("ExamPle.Com"))
    }

}
