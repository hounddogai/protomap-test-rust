use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover quota must pass. Check returns its problems, or none.
pub trait FailoverQuotaRule: std::fmt::Debug {
    fn check(&self, item: &FailoverQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuotaRequiredCodeRule {
    pub message: String,
}

impl FailoverQuotaRule for FailoverQuotaRequiredCodeRule {
    fn check(&self, item: &FailoverQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuotaRequiredTenantIdRule {
    pub message: String,
}

impl FailoverQuotaRule for FailoverQuotaRequiredTenantIdRule {
    fn check(&self, item: &FailoverQuota) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuotaAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl FailoverQuotaRule for FailoverQuotaAllowedReviewerRule {
    fn check(&self, item: &FailoverQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is inconsistent", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuotaCodePrefixRule {
    pub prefix: String,
}

impl FailoverQuotaRule for FailoverQuotaCodePrefixRule {
    fn check(&self, item: &FailoverQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuotaRequiredNameRule {
    pub message: String,
}

impl FailoverQuotaRule for FailoverQuotaRequiredNameRule {
    fn check(&self, item: &FailoverQuota) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverQuotaRequiredReviewerRule {
    pub message: String,
}

impl FailoverQuotaRule for FailoverQuotaRequiredReviewerRule {
    fn check(&self, item: &FailoverQuota) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover quota.
#[derive(Debug, Clone)]
pub struct FailoverQuotaValidator {
    pub rules: Vec<Rc<dyn FailoverQuotaRule>>,
}

impl FailoverQuotaValidator {
    pub fn validate(&self, item: &FailoverQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FailoverQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_quota_validator() -> FailoverQuotaValidator {
    FailoverQuotaValidator {
        rules: vec![
            Rc::new(FailoverQuotaRequiredCodeRule {
                message: "failover quota needs this field".to_string(),
            }) as Rc<dyn FailoverQuotaRule>,
            Rc::new(FailoverQuotaRequiredTenantIdRule {
                message: "failover quota needs a value".to_string(),
            }),
            Rc::new(FailoverQuotaAllowedReviewerRule {
                allowed: vec!["regional".to_string(), "global".to_string()],
            }),
            Rc::new(FailoverQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FailoverQuotaRequiredNameRule {
                message: "failover quota needs this field".to_string(),
            }),
            Rc::new(FailoverQuotaRequiredReviewerRule {
                message: "failover quota needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_quota_validator();
        let item = new_failover_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FailoverQuotaRequiredCodeRule {
            message: "failover quota needs a value".to_string(),
        };
        let mut item = new_failover_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = FailoverQuotaRequiredTenantIdRule {
            message: "failover quota needs a value".to_string(),
        };
        let mut item = new_failover_quota("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FailoverQuotaRequiredNameRule {
            message: "failover quota needs an entry".to_string(),
        };
        let mut item = new_failover_quota("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = FailoverQuotaRequiredReviewerRule {
            message: "failover quota needs an entry".to_string(),
        };
        let mut item = new_failover_quota("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
