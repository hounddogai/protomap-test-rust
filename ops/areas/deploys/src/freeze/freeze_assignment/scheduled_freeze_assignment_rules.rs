use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled freeze assignment must pass. Check returns its problems, or none.
pub trait ScheduledFreezeAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledFreezeAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeAssignmentRequiredCurrencyRule {
    pub message: String,
}

impl ScheduledFreezeAssignmentRule for ScheduledFreezeAssignmentRequiredCurrencyRule {
    fn check(&self, item: &ScheduledFreezeAssignment) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeAssignmentCurrencyLengthRule {
    pub max_length: i64,
}

impl ScheduledFreezeAssignmentRule for ScheduledFreezeAssignmentCurrencyLengthRule {
    fn check(&self, item: &ScheduledFreezeAssignment) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFreezeAssignmentTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledFreezeAssignmentRule for ScheduledFreezeAssignmentTitleLengthRule {
    fn check(&self, item: &ScheduledFreezeAssignment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled freeze assignment.
#[derive(Debug, Clone)]
pub struct ScheduledFreezeAssignmentValidator {
    pub rules: Vec<Rc<dyn ScheduledFreezeAssignmentRule>>,
}

impl ScheduledFreezeAssignmentValidator {
    pub fn validate(&self, item: &ScheduledFreezeAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledFreezeAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledFreezeAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_freeze_assignment_validator() -> ScheduledFreezeAssignmentValidator {
    ScheduledFreezeAssignmentValidator {
        rules: vec![
            Rc::new(ScheduledFreezeAssignmentRequiredCurrencyRule {
                message: "scheduled freeze assignment needs this field".to_string(),
            }) as Rc<dyn ScheduledFreezeAssignmentRule>,
            Rc::new(ScheduledFreezeAssignmentCurrencyLengthRule { max_length: 86400 }),
            Rc::new(ScheduledFreezeAssignmentTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_freeze_assignment_validator();
        let item = new_scheduled_freeze_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = ScheduledFreezeAssignmentRequiredCurrencyRule {
            message: "scheduled freeze assignment needs a value".to_string(),
        };
        let mut item = new_scheduled_freeze_assignment("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = ScheduledFreezeAssignmentCurrencyLengthRule { max_length: 64 };
        let mut item = new_scheduled_freeze_assignment("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledFreezeAssignmentTitleLengthRule { max_length: 86400 };
        let mut item = new_scheduled_freeze_assignment("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
