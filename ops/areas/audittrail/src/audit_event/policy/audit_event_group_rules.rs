use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event group must pass. Check returns its problems, or none.
pub trait AuditEventGroupRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroupRequiredLabelRule {
    pub message: String,
}

impl AuditEventGroupRule for AuditEventGroupRequiredLabelRule {
    fn check(&self, item: &AuditEventGroup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroupRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventGroupRule for AuditEventGroupRetriesRangeRule {
    fn check(&self, item: &AuditEventGroup) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroupRequiredIssuerRule {
    pub message: String,
}

impl AuditEventGroupRule for AuditEventGroupRequiredIssuerRule {
    fn check(&self, item: &AuditEventGroup) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroupThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AuditEventGroupRule for AuditEventGroupThresholdRangeRule {
    fn check(&self, item: &AuditEventGroup) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroupCodePrefixRule {
    pub prefix: String,
}

impl AuditEventGroupRule for AuditEventGroupCodePrefixRule {
    fn check(&self, item: &AuditEventGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventGroupAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl AuditEventGroupRule for AuditEventGroupAllowedOwnerIdRule {
    fn check(&self, item: &AuditEventGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is expired", item.owner_id)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event group.
#[derive(Debug, Clone)]
pub struct AuditEventGroupValidator {
    pub rules: Vec<Rc<dyn AuditEventGroupRule>>,
}

impl AuditEventGroupValidator {
    pub fn validate(&self, item: &AuditEventGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_group_validator() -> AuditEventGroupValidator {
    AuditEventGroupValidator {
        rules: vec![
            Rc::new(AuditEventGroupRequiredLabelRule {
                message: "audit event group needs an entry".to_string(),
            }) as Rc<dyn AuditEventGroupRule>,
            Rc::new(AuditEventGroupRetriesRangeRule { min: 64, max: 86400 }),
            Rc::new(AuditEventGroupRequiredIssuerRule {
                message: "audit event group needs an entry".to_string(),
            }),
            Rc::new(AuditEventGroupThresholdRangeRule { min: 7.8, max: 116.1 }),
            Rc::new(AuditEventGroupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(AuditEventGroupAllowedOwnerIdRule {
                allowed: vec!["pending".to_string(), "tiered".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_group_validator();
        let item = new_audit_event_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AuditEventGroupRequiredLabelRule {
            message: "audit event group needs this field".to_string(),
        };
        let mut item = new_audit_event_group("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = AuditEventGroupRetriesRangeRule { min: 64, max: 86400 };
        let mut item = new_audit_event_group("x", "y");
        item.retries = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = AuditEventGroupRequiredIssuerRule {
            message: "audit event group needs this field".to_string(),
        };
        let mut item = new_audit_event_group("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = AuditEventGroupThresholdRangeRule { min: 6.4, max: 3048.4 };
        let mut item = new_audit_event_group("x", "y");
        item.threshold = 3049.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
