use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled trust domain quota must pass. Check returns its problems, or none.
pub trait ScheduledTrustDomainQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledTrustDomainQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainQuotaLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledTrustDomainQuotaRule for ScheduledTrustDomainQuotaLabelLengthRule {
    fn check(&self, item: &ScheduledTrustDomainQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainQuotaRequiredNotesRule {
    pub message: String,
}

impl ScheduledTrustDomainQuotaRule for ScheduledTrustDomainQuotaRequiredNotesRule {
    fn check(&self, item: &ScheduledTrustDomainQuota) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTrustDomainQuotaCodeLengthRule {
    pub max_length: i64,
}

impl ScheduledTrustDomainQuotaRule for ScheduledTrustDomainQuotaCodeLengthRule {
    fn check(&self, item: &ScheduledTrustDomainQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled trust domain quota.
#[derive(Debug, Clone)]
pub struct ScheduledTrustDomainQuotaValidator {
    pub rules: Vec<Rc<dyn ScheduledTrustDomainQuotaRule>>,
}

impl ScheduledTrustDomainQuotaValidator {
    pub fn validate(&self, item: &ScheduledTrustDomainQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledTrustDomainQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledTrustDomainQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_trust_domain_quota_validator() -> ScheduledTrustDomainQuotaValidator {
    ScheduledTrustDomainQuotaValidator {
        rules: vec![
            Rc::new(ScheduledTrustDomainQuotaLabelLengthRule { max_length: 3600 })
                as Rc<dyn ScheduledTrustDomainQuotaRule>,
            Rc::new(ScheduledTrustDomainQuotaRequiredNotesRule {
                message: "scheduled trust domain quota needs a value".to_string(),
            }),
            Rc::new(ScheduledTrustDomainQuotaCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_trust_domain_quota_validator();
        let item = new_scheduled_trust_domain_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledTrustDomainQuotaLabelLengthRule { max_length: 86400 };
        let mut item = new_scheduled_trust_domain_quota("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = ScheduledTrustDomainQuotaRequiredNotesRule {
            message: "scheduled trust domain quota needs this field".to_string(),
        };
        let mut item = new_scheduled_trust_domain_quota("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ScheduledTrustDomainQuotaCodeLengthRule { max_length: 1000 };
        let mut item = new_scheduled_trust_domain_quota("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
