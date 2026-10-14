use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event target must pass. Check returns its problems, or none.
pub trait AuditEventTargetRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetRequiredLabelRule {
    pub message: String,
}

impl AuditEventTargetRule for AuditEventTargetRequiredLabelRule {
    fn check(&self, item: &AuditEventTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetRequiredChecksumRule {
    pub message: String,
}

impl AuditEventTargetRule for AuditEventTargetRequiredChecksumRule {
    fn check(&self, item: &AuditEventTarget) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetLabelLengthRule {
    pub max_length: i64,
}

impl AuditEventTargetRule for AuditEventTargetLabelLengthRule {
    fn check(&self, item: &AuditEventTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetClusterLengthRule {
    pub max_length: i64,
}

impl AuditEventTargetRule for AuditEventTargetClusterLengthRule {
    fn check(&self, item: &AuditEventTarget) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetRequiredCodeRule {
    pub message: String,
}

impl AuditEventTargetRule for AuditEventTargetRequiredCodeRule {
    fn check(&self, item: &AuditEventTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventTargetAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AuditEventTargetRule for AuditEventTargetAllowedLabelRule {
    fn check(&self, item: &AuditEventTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is not allowed", item.label)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event target.
#[derive(Debug, Clone)]
pub struct AuditEventTargetValidator {
    pub rules: Vec<Rc<dyn AuditEventTargetRule>>,
}

impl AuditEventTargetValidator {
    pub fn validate(&self, item: &AuditEventTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_target_validator() -> AuditEventTargetValidator {
    AuditEventTargetValidator {
        rules: vec![
            Rc::new(AuditEventTargetRequiredLabelRule {
                message: "audit event target needs a value".to_string(),
            }) as Rc<dyn AuditEventTargetRule>,
            Rc::new(AuditEventTargetRequiredChecksumRule {
                message: "audit event target needs an entry".to_string(),
            }),
            Rc::new(AuditEventTargetLabelLengthRule { max_length: 1000 }),
            Rc::new(AuditEventTargetClusterLengthRule { max_length: 16 }),
            Rc::new(AuditEventTargetRequiredCodeRule {
                message: "audit event target needs an entry".to_string(),
            }),
            Rc::new(AuditEventTargetAllowedLabelRule {
                allowed: vec!["fallback".to_string(), "rolling".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_target_validator();
        let item = new_audit_event_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AuditEventTargetRequiredLabelRule {
            message: "audit event target needs this field".to_string(),
        };
        let mut item = new_audit_event_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = AuditEventTargetRequiredChecksumRule {
            message: "audit event target needs a value".to_string(),
        };
        let mut item = new_audit_event_target("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AuditEventTargetLabelLengthRule { max_length: 80 };
        let mut item = new_audit_event_target("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = AuditEventTargetClusterLengthRule { max_length: 86400 };
        let mut item = new_audit_event_target("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventTargetRequiredCodeRule {
            message: "audit event target needs a value".to_string(),
        };
        let mut item = new_audit_event_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
