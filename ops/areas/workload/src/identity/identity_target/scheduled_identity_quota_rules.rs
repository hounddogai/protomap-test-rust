use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled identity quota must pass. Check returns its problems, or none.
pub trait ScheduledIdentityQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledIdentityQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityQuotaAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl ScheduledIdentityQuotaRule for ScheduledIdentityQuotaAllowedFingerprintRule {
    fn check(&self, item: &ScheduledIdentityQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is below minimum", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityQuotaRequiredTitleRule {
    pub message: String,
}

impl ScheduledIdentityQuotaRule for ScheduledIdentityQuotaRequiredTitleRule {
    fn check(&self, item: &ScheduledIdentityQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityQuotaCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledIdentityQuotaRule for ScheduledIdentityQuotaCodeLengthRule {
    fn check(&self, item: &ScheduledIdentityQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledIdentityQuotaTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledIdentityQuotaRule for ScheduledIdentityQuotaTitleLengthRule {
    fn check(&self, item: &ScheduledIdentityQuota) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled identity quota.
#[derive(Debug, Clone)]
pub struct ScheduledIdentityQuotaValidator {
    pub rules: Vec<Rc<dyn ScheduledIdentityQuotaRule>>,
}

impl ScheduledIdentityQuotaValidator {
    pub fn validate(&self, item: &ScheduledIdentityQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledIdentityQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledIdentityQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_identity_quota_validator() -> ScheduledIdentityQuotaValidator {
    ScheduledIdentityQuotaValidator {
        rules: vec![
            Rc::new(ScheduledIdentityQuotaAllowedFingerprintRule {
                allowed: vec!["pending".to_string()],
            }) as Rc<dyn ScheduledIdentityQuotaRule>,
            Rc::new(ScheduledIdentityQuotaRequiredTitleRule {
                message: "scheduled identity quota needs this field".to_string(),
            }),
            Rc::new(ScheduledIdentityQuotaCodeLengthRule { max_length: 16 }),
            Rc::new(ScheduledIdentityQuotaTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_identity_quota_validator();
        let item = new_scheduled_identity_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledIdentityQuotaRequiredTitleRule {
            message: "scheduled identity quota needs a value".to_string(),
        };
        let mut item = new_scheduled_identity_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledIdentityQuotaCodeLengthRule { max_length: 1000 };
        let mut item = new_scheduled_identity_quota("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledIdentityQuotaTitleLengthRule { max_length: 255 };
        let mut item = new_scheduled_identity_quota("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
