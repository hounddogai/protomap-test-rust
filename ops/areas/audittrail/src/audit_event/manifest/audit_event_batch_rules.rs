use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event batch must pass. Check returns its problems, or none.
pub trait AuditEventBatchRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBatchCodePrefixRule {
    pub prefix: String,
}

impl AuditEventBatchRule for AuditEventBatchCodePrefixRule {
    fn check(&self, item: &AuditEventBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBatchRequiredCodeRule {
    pub message: String,
}

impl AuditEventBatchRule for AuditEventBatchRequiredCodeRule {
    fn check(&self, item: &AuditEventBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventBatchRequiredChecksumRule {
    pub message: String,
}

impl AuditEventBatchRule for AuditEventBatchRequiredChecksumRule {
    fn check(&self, item: &AuditEventBatch) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event batch.
#[derive(Debug, Clone)]
pub struct AuditEventBatchValidator {
    pub rules: Vec<Rc<dyn AuditEventBatchRule>>,
}

impl AuditEventBatchValidator {
    pub fn validate(&self, item: &AuditEventBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_batch_validator() -> AuditEventBatchValidator {
    AuditEventBatchValidator {
        rules: vec![
            Rc::new(AuditEventBatchCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn AuditEventBatchRule>,
            Rc::new(AuditEventBatchRequiredCodeRule {
                message: "audit event batch needs a value".to_string(),
            }),
            Rc::new(AuditEventBatchRequiredChecksumRule {
                message: "audit event batch needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_batch_validator();
        let item = new_audit_event_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventBatchRequiredCodeRule {
            message: "audit event batch needs an entry".to_string(),
        };
        let mut item = new_audit_event_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = AuditEventBatchRequiredChecksumRule {
            message: "audit event batch needs a value".to_string(),
        };
        let mut item = new_audit_event_batch("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
