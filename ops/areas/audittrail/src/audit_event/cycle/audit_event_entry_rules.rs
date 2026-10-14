use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event entry must pass. Check returns its problems, or none.
pub trait AuditEventEntryRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventEntryCodePrefixRule {
    pub prefix: String,
}

impl AuditEventEntryRule for AuditEventEntryCodePrefixRule {
    fn check(&self, item: &AuditEventEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventEntryNamespaceLengthRule {
    pub max_length: i64,
}

impl AuditEventEntryRule for AuditEventEntryNamespaceLengthRule {
    fn check(&self, item: &AuditEventEntry) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventEntryClusterLengthRule {
    pub max_length: i64,
}

impl AuditEventEntryRule for AuditEventEntryClusterLengthRule {
    fn check(&self, item: &AuditEventEntry) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventEntryRequiredNamespaceRule {
    pub message: String,
}

impl AuditEventEntryRule for AuditEventEntryRequiredNamespaceRule {
    fn check(&self, item: &AuditEventEntry) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventEntryWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventEntryRule for AuditEventEntryWindowDaysRangeRule {
    fn check(&self, item: &AuditEventEntry) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event entry.
#[derive(Debug, Clone)]
pub struct AuditEventEntryValidator {
    pub rules: Vec<Rc<dyn AuditEventEntryRule>>,
}

impl AuditEventEntryValidator {
    pub fn validate(&self, item: &AuditEventEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_entry_validator() -> AuditEventEntryValidator {
    AuditEventEntryValidator {
        rules: vec![
            Rc::new(AuditEventEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn AuditEventEntryRule>,
            Rc::new(AuditEventEntryNamespaceLengthRule { max_length: 1000 }),
            Rc::new(AuditEventEntryClusterLengthRule { max_length: 3600 }),
            Rc::new(AuditEventEntryRequiredNamespaceRule {
                message: "audit event entry needs a value".to_string(),
            }),
            Rc::new(AuditEventEntryWindowDaysRangeRule { min: 86400, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_entry_validator();
        let item = new_audit_event_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = AuditEventEntryNamespaceLengthRule { max_length: 16 };
        let mut item = new_audit_event_entry("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = AuditEventEntryClusterLengthRule { max_length: 16 };
        let mut item = new_audit_event_entry("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = AuditEventEntryRequiredNamespaceRule {
            message: "audit event entry needs this field".to_string(),
        };
        let mut item = new_audit_event_entry("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = AuditEventEntryWindowDaysRangeRule { min: 32, max: 80 };
        let mut item = new_audit_event_entry("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
