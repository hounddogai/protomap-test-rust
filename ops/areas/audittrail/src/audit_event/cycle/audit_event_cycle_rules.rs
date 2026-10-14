use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a audit event cycle must pass. Check returns its problems, or none.
pub trait AuditEventCycleRule: std::fmt::Debug {
    fn check(&self, item: &AuditEventCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleRequiredCodeRule {
    pub message: String,
}

impl AuditEventCycleRule for AuditEventCycleRequiredCodeRule {
    fn check(&self, item: &AuditEventCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleCodeLengthRule {
    pub max_length: i64,
}

impl AuditEventCycleRule for AuditEventCycleCodeLengthRule {
    fn check(&self, item: &AuditEventCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AuditEventCycleRule for AuditEventCycleSequenceRangeRule {
    fn check(&self, item: &AuditEventCycle) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleRequiredRegionRule {
    pub message: String,
}

impl AuditEventCycleRule for AuditEventCycleRequiredRegionRule {
    fn check(&self, item: &AuditEventCycle) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleTitleLengthRule {
    pub max_length: i64,
}

impl AuditEventCycleRule for AuditEventCycleTitleLengthRule {
    fn check(&self, item: &AuditEventCycle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventCycleRequiredTitleRule {
    pub message: String,
}

impl AuditEventCycleRule for AuditEventCycleRequiredTitleRule {
    fn check(&self, item: &AuditEventCycle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a audit event cycle.
#[derive(Debug, Clone)]
pub struct AuditEventCycleValidator {
    pub rules: Vec<Rc<dyn AuditEventCycleRule>>,
}

impl AuditEventCycleValidator {
    pub fn validate(&self, item: &AuditEventCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AuditEventCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AuditEventCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_audit_event_cycle_validator() -> AuditEventCycleValidator {
    AuditEventCycleValidator {
        rules: vec![
            Rc::new(AuditEventCycleRequiredCodeRule {
                message: "audit event cycle needs this field".to_string(),
            }) as Rc<dyn AuditEventCycleRule>,
            Rc::new(AuditEventCycleCodeLengthRule { max_length: 86400 }),
            Rc::new(AuditEventCycleSequenceRangeRule { min: 1000, max: 86400 }),
            Rc::new(AuditEventCycleRequiredRegionRule {
                message: "audit event cycle needs a value".to_string(),
            }),
            Rc::new(AuditEventCycleTitleLengthRule { max_length: 255 }),
            Rc::new(AuditEventCycleRequiredTitleRule {
                message: "audit event cycle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_audit_event_cycle_validator();
        let item = new_audit_event_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AuditEventCycleRequiredCodeRule {
            message: "audit event cycle needs a value".to_string(),
        };
        let mut item = new_audit_event_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AuditEventCycleCodeLengthRule { max_length: 16 };
        let mut item = new_audit_event_cycle("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = AuditEventCycleSequenceRangeRule { min: 64, max: 80 };
        let mut item = new_audit_event_cycle("x", "y");
        item.sequence = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = AuditEventCycleRequiredRegionRule {
            message: "audit event cycle needs this field".to_string(),
        };
        let mut item = new_audit_event_cycle("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AuditEventCycleTitleLengthRule { max_length: 32 };
        let mut item = new_audit_event_cycle("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AuditEventCycleRequiredTitleRule {
            message: "audit event cycle needs a value".to_string(),
        };
        let mut item = new_audit_event_cycle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
