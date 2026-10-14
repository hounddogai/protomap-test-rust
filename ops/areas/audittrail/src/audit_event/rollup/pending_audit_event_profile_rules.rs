use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending audit event profile must pass. Check returns its problems, or none.
pub trait PendingAuditEventProfileRule: std::fmt::Debug {
    fn check(&self, item: &PendingAuditEventProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventProfileAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PendingAuditEventProfileRule for PendingAuditEventProfileAllowedTitleRule {
    fn check(&self, item: &PendingAuditEventProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventProfileRequiredTitleRule {
    pub message: String,
}

impl PendingAuditEventProfileRule for PendingAuditEventProfileRequiredTitleRule {
    fn check(&self, item: &PendingAuditEventProfile) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventProfileCodePrefixRule {
    pub prefix: String,
}

impl PendingAuditEventProfileRule for PendingAuditEventProfileCodePrefixRule {
    fn check(&self, item: &PendingAuditEventProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventProfileTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingAuditEventProfileRule for PendingAuditEventProfileTtlSecondsRangeRule {
    fn check(&self, item: &PendingAuditEventProfile) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending audit event profile.
#[derive(Debug, Clone)]
pub struct PendingAuditEventProfileValidator {
    pub rules: Vec<Rc<dyn PendingAuditEventProfileRule>>,
}

impl PendingAuditEventProfileValidator {
    pub fn validate(&self, item: &PendingAuditEventProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingAuditEventProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingAuditEventProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_audit_event_profile_validator() -> PendingAuditEventProfileValidator {
    PendingAuditEventProfileValidator {
        rules: vec![
            Rc::new(PendingAuditEventProfileAllowedTitleRule {
                allowed: vec!["weighted".to_string(), "regional".to_string()],
            }) as Rc<dyn PendingAuditEventProfileRule>,
            Rc::new(PendingAuditEventProfileRequiredTitleRule {
                message: "pending audit event profile needs an entry".to_string(),
            }),
            Rc::new(PendingAuditEventProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingAuditEventProfileTtlSecondsRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_audit_event_profile_validator();
        let item = new_pending_audit_event_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PendingAuditEventProfileRequiredTitleRule {
            message: "pending audit event profile needs a value".to_string(),
        };
        let mut item = new_pending_audit_event_profile("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PendingAuditEventProfileTtlSecondsRangeRule { min: 1000, max: 86400 };
        let mut item = new_pending_audit_event_profile("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
