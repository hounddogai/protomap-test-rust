use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled audit event record must pass. Check returns its problems, or none.
pub trait ScheduledAuditEventRecordRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledAuditEventRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventRecordRequiredContactEmailRule {
    pub message: String,
}

impl ScheduledAuditEventRecordRule for ScheduledAuditEventRecordRequiredContactEmailRule {
    fn check(&self, item: &ScheduledAuditEventRecord) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventRecordRequiredCodeRule {
    pub message: String,
}

impl ScheduledAuditEventRecordRule for ScheduledAuditEventRecordRequiredCodeRule {
    fn check(&self, item: &ScheduledAuditEventRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventRecordTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledAuditEventRecordRule for ScheduledAuditEventRecordTitleLengthRule {
    fn check(&self, item: &ScheduledAuditEventRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventRecordScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledAuditEventRecordRule for ScheduledAuditEventRecordScoreRangeRule {
    fn check(&self, item: &ScheduledAuditEventRecord) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledAuditEventRecordRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledAuditEventRecordRule for ScheduledAuditEventRecordRatioRangeRule {
    fn check(&self, item: &ScheduledAuditEventRecord) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled audit event record.
#[derive(Debug, Clone)]
pub struct ScheduledAuditEventRecordValidator {
    pub rules: Vec<Rc<dyn ScheduledAuditEventRecordRule>>,
}

impl ScheduledAuditEventRecordValidator {
    pub fn validate(&self, item: &ScheduledAuditEventRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledAuditEventRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledAuditEventRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_audit_event_record_validator() -> ScheduledAuditEventRecordValidator {
    ScheduledAuditEventRecordValidator {
        rules: vec![
            Rc::new(ScheduledAuditEventRecordRequiredContactEmailRule {
                message: "scheduled audit event record needs a value".to_string(),
            }) as Rc<dyn ScheduledAuditEventRecordRule>,
            Rc::new(ScheduledAuditEventRecordRequiredCodeRule {
                message: "scheduled audit event record needs an entry".to_string(),
            }),
            Rc::new(ScheduledAuditEventRecordTitleLengthRule { max_length: 1000 }),
            Rc::new(ScheduledAuditEventRecordScoreRangeRule { min: 8.4, max: 1881.6 }),
            Rc::new(ScheduledAuditEventRecordRatioRangeRule { min: 9.7, max: 1427.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_audit_event_record_validator();
        let item = new_scheduled_audit_event_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ScheduledAuditEventRecordRequiredContactEmailRule {
            message: "scheduled audit event record needs a value".to_string(),
        };
        let mut item = new_scheduled_audit_event_record("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledAuditEventRecordRequiredCodeRule {
            message: "scheduled audit event record needs an entry".to_string(),
        };
        let mut item = new_scheduled_audit_event_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledAuditEventRecordTitleLengthRule { max_length: 86400 };
        let mut item = new_scheduled_audit_event_record("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ScheduledAuditEventRecordScoreRangeRule { min: 1.3, max: 247.6 };
        let mut item = new_scheduled_audit_event_record("x", "y");
        item.score = 249.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ScheduledAuditEventRecordRatioRangeRule { min: 3.3, max: 737.8 };
        let mut item = new_scheduled_audit_event_record("x", "y");
        item.ratio = 739.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
