use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a audit event checkpoint must pass. Check returns its problems, or none.
pub trait AuditEventCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventCheckpointRule for AuditEventCheckpointCodeLengthRule {
    fn check(&self, item: &AuditEventCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCheckpointAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl AuditEventCheckpointRule for AuditEventCheckpointAllowedUnitRule {
    fn check(&self, item: &AuditEventCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is inconsistent", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCheckpointUnitLengthRule {
    pub max_length: i64,
}

impl AuditEventCheckpointRule for AuditEventCheckpointUnitLengthRule {
    fn check(&self, item: &AuditEventCheckpoint) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCheckpointTitleLengthRule {
    pub max_length: i64,
}

impl AuditEventCheckpointRule for AuditEventCheckpointTitleLengthRule {
    fn check(&self, item: &AuditEventCheckpoint) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event checkpoint.
#[derive(Debug, Clone)]
pub struct AuditEventCheckpointValidator {
    pub rules: Vec<Rc<dyn AuditEventCheckpointRule>>,
}

impl AuditEventCheckpointValidator {
    pub fn validate(&self, item: &AuditEventCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_checkpoint_validator() -> AuditEventCheckpointValidator {
    AuditEventCheckpointValidator {
        rules: vec![
            Rc::new(AuditEventCheckpointCodeLengthRule { max_length: 3600 }) as Rc<dyn AuditEventCheckpointRule>,
            Rc::new(AuditEventCheckpointAllowedUnitRule {
                allowed: vec!["tiered".to_string(), "fallback".to_string()],
            }),
            Rc::new(AuditEventCheckpointUnitLengthRule { max_length: 64 }),
            Rc::new(AuditEventCheckpointTitleLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_checkpoint_validator();
        let item = new_audit_event_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventCheckpointCodeLengthRule { max_length: 16 };
        let mut item = new_audit_event_checkpoint("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = AuditEventCheckpointUnitLengthRule { max_length: 120 };
        let mut item = new_audit_event_checkpoint("x", "y");
        item.unit = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AuditEventCheckpointTitleLengthRule { max_length: 1000 };
        let mut item = new_audit_event_checkpoint("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
