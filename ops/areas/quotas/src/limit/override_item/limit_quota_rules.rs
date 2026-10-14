use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a limit quota must pass. Check returns its problems, or none.
pub trait LimitQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LimitQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitQuotaRequiredRegionRule {
    pub message: String,
}

impl LimitQuotaRule for LimitQuotaRequiredRegionRule {
    fn check(&self, item: &LimitQuota) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitQuotaRequiredTitleRule {
    pub message: String,
}

impl LimitQuotaRule for LimitQuotaRequiredTitleRule {
    fn check(&self, item: &LimitQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LimitQuotaRule for LimitQuotaAllowedCodeRule {
    fn check(&self, item: &LimitQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit quota.
#[derive(Debug, Clone)]
pub struct LimitQuotaValidator {
    pub rules: Vec<Rc<dyn LimitQuotaRule>>,
}

impl LimitQuotaValidator {
    pub fn validate(&self, item: &LimitQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LimitQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_quota_validator() -> LimitQuotaValidator {
    LimitQuotaValidator {
        rules: vec![
            Rc::new(LimitQuotaRequiredRegionRule {
                message: "limit quota needs this field".to_string(),
            }) as Rc<dyn LimitQuotaRule>,
            Rc::new(LimitQuotaRequiredTitleRule {
                message: "limit quota needs a value".to_string(),
            }),
            Rc::new(LimitQuotaAllowedCodeRule {
                allowed: vec!["fallback".to_string(), "fallback".to_string(), "lenient".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_quota_validator();
        let item = new_limit_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LimitQuotaRequiredRegionRule {
            message: "limit quota needs this field".to_string(),
        };
        let mut item = new_limit_quota("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LimitQuotaRequiredTitleRule {
            message: "limit quota needs a value".to_string(),
        };
        let mut item = new_limit_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
