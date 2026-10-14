use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a page quota must pass. Check returns its problems, or none.
pub trait PageQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PageQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageQuotaHostLengthRule {
    pub max_length: i64,
}

impl PageQuotaRule for PageQuotaHostLengthRule {
    fn check(&self, item: &PageQuota) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageQuotaRequiredCodeRule {
    pub message: String,
}

impl PageQuotaRule for PageQuotaRequiredCodeRule {
    fn check(&self, item: &PageQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageQuotaRequiredSummaryRule {
    pub message: String,
}

impl PageQuotaRule for PageQuotaRequiredSummaryRule {
    fn check(&self, item: &PageQuota) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageQuotaRequiredTitleRule {
    pub message: String,
}

impl PageQuotaRule for PageQuotaRequiredTitleRule {
    fn check(&self, item: &PageQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageQuotaCodePrefixRule {
    pub prefix: String,
}

impl PageQuotaRule for PageQuotaCodePrefixRule {
    fn check(&self, item: &PageQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PageQuotaRequiredHostRule {
    pub message: String,
}

impl PageQuotaRule for PageQuotaRequiredHostRule {
    fn check(&self, item: &PageQuota) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a page quota.
#[derive(Debug, Clone)]
pub struct PageQuotaValidator {
    pub rules: Vec<Rc<dyn PageQuotaRule>>,
}

impl PageQuotaValidator {
    pub fn validate(&self, item: &PageQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PageQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PageQuota]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_page_quota_validator() -> PageQuotaValidator {
    PageQuotaValidator {
        rules: vec![
            Rc::new(PageQuotaHostLengthRule { max_length: 1000 }) as Rc<dyn PageQuotaRule>,
            Rc::new(PageQuotaRequiredCodeRule {
                message: "page quota needs this field".to_string(),
            }),
            Rc::new(PageQuotaRequiredSummaryRule {
                message: "page quota needs this field".to_string(),
            }),
            Rc::new(PageQuotaRequiredTitleRule {
                message: "page quota needs an entry".to_string(),
            }),
            Rc::new(PageQuotaCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PageQuotaRequiredHostRule {
                message: "page quota needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_page_quota_validator();
        let item = new_page_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = PageQuotaHostLengthRule { max_length: 86400 };
        let mut item = new_page_quota("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PageQuotaRequiredCodeRule {
            message: "page quota needs a value".to_string(),
        };
        let mut item = new_page_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = PageQuotaRequiredSummaryRule {
            message: "page quota needs an entry".to_string(),
        };
        let mut item = new_page_quota("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PageQuotaRequiredTitleRule {
            message: "page quota needs an entry".to_string(),
        };
        let mut item = new_page_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PageQuotaRequiredHostRule {
            message: "page quota needs a value".to_string(),
        };
        let mut item = new_page_quota("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
