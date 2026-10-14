use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event snapshot must pass. Check returns its problems, or none.
pub trait AuditEventSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSnapshotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventSnapshotRule for AuditEventSnapshotRevisionRangeRule {
    fn check(&self, item: &AuditEventSnapshot) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSnapshotRequiredCodeRule {
    pub message: String,
}

impl AuditEventSnapshotRule for AuditEventSnapshotRequiredCodeRule {
    fn check(&self, item: &AuditEventSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl AuditEventSnapshotRule for AuditEventSnapshotLabelLengthRule {
    fn check(&self, item: &AuditEventSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event snapshot.
#[derive(Debug, Clone)]
pub struct AuditEventSnapshotValidator {
    pub rules: Vec<Rc<dyn AuditEventSnapshotRule>>,
}

impl AuditEventSnapshotValidator {
    pub fn validate(&self, item: &AuditEventSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AuditEventSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_snapshot_validator() -> AuditEventSnapshotValidator {
    AuditEventSnapshotValidator {
        rules: vec![
            Rc::new(AuditEventSnapshotRevisionRangeRule { min: 32, max: 3600 }) as Rc<dyn AuditEventSnapshotRule>,
            Rc::new(AuditEventSnapshotRequiredCodeRule {
                message: "audit event snapshot needs this field".to_string(),
            }),
            Rc::new(AuditEventSnapshotLabelLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_snapshot_validator();
        let item = new_audit_event_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = AuditEventSnapshotRevisionRangeRule { min: 32, max: 32 };
        let mut item = new_audit_event_snapshot("x", "y");
        item.revision = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventSnapshotRequiredCodeRule {
            message: "audit event snapshot needs a value".to_string(),
        };
        let mut item = new_audit_event_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AuditEventSnapshotLabelLengthRule { max_length: 1000 };
        let mut item = new_audit_event_snapshot("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
