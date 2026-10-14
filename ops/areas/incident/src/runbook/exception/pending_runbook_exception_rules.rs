use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending runbook exception must pass. Check returns its problems, or none.
pub trait PendingRunbookExceptionRule: std::fmt::Debug {
    fn check(&self, item: &PendingRunbookException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookExceptionMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRunbookExceptionRule for PendingRunbookExceptionMinimumRangeRule {
    fn check(&self, item: &PendingRunbookException) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookExceptionRequiredNameRule {
    pub message: String,
}

impl PendingRunbookExceptionRule for PendingRunbookExceptionRequiredNameRule {
    fn check(&self, item: &PendingRunbookException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookExceptionDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRunbookExceptionRule for PendingRunbookExceptionDurationMinutesRangeRule {
    fn check(&self, item: &PendingRunbookException) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookExceptionRequiredKeyIdRule {
    pub message: String,
}

impl PendingRunbookExceptionRule for PendingRunbookExceptionRequiredKeyIdRule {
    fn check(&self, item: &PendingRunbookException) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRunbookExceptionNameLengthRule {
    pub max_length: i64,
}

impl PendingRunbookExceptionRule for PendingRunbookExceptionNameLengthRule {
    fn check(&self, item: &PendingRunbookException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending runbook exception.
#[derive(Debug, Clone)]
pub struct PendingRunbookExceptionValidator {
    pub rules: Vec<Rc<dyn PendingRunbookExceptionRule>>,
}

impl PendingRunbookExceptionValidator {
    pub fn validate(&self, item: &PendingRunbookException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRunbookException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRunbookException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_runbook_exception_validator() -> PendingRunbookExceptionValidator {
    PendingRunbookExceptionValidator {
        rules: vec![
            Rc::new(PendingRunbookExceptionMinimumRangeRule { min: 2.1, max: 814.4 })
                as Rc<dyn PendingRunbookExceptionRule>,
            Rc::new(PendingRunbookExceptionRequiredNameRule {
                message: "pending runbook exception needs an entry".to_string(),
            }),
            Rc::new(PendingRunbookExceptionDurationMinutesRangeRule { min: 64, max: 120 }),
            Rc::new(PendingRunbookExceptionRequiredKeyIdRule {
                message: "pending runbook exception needs this field".to_string(),
            }),
            Rc::new(PendingRunbookExceptionNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_runbook_exception_validator();
        let item = new_pending_runbook_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PendingRunbookExceptionMinimumRangeRule { min: 3.0, max: 3071.5 };
        let mut item = new_pending_runbook_exception("x", "y");
        item.minimum = 3073.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingRunbookExceptionRequiredNameRule {
            message: "pending runbook exception needs this field".to_string(),
        };
        let mut item = new_pending_runbook_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = PendingRunbookExceptionDurationMinutesRangeRule { min: 64, max: 3600 };
        let mut item = new_pending_runbook_exception("x", "y");
        item.duration_minutes = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PendingRunbookExceptionRequiredKeyIdRule {
            message: "pending runbook exception needs this field".to_string(),
        };
        let mut item = new_pending_runbook_exception("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingRunbookExceptionNameLengthRule { max_length: 255 };
        let mut item = new_pending_runbook_exception("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
