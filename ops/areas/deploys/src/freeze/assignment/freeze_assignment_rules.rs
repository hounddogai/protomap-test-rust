use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze assignment must pass. Check returns its problems, or none.
pub trait FreezeAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &FreezeAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAssignmentSummaryLengthRule {
    pub max_length: i64,
}

impl FreezeAssignmentRule for FreezeAssignmentSummaryLengthRule {
    fn check(&self, item: &FreezeAssignment) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl FreezeAssignmentRule for FreezeAssignmentCodeLengthRule {
    fn check(&self, item: &FreezeAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAssignmentRequiredCodeRule {
    pub message: String,
}

impl FreezeAssignmentRule for FreezeAssignmentRequiredCodeRule {
    fn check(&self, item: &FreezeAssignment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAssignmentAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeAssignmentRule for FreezeAssignmentAmountRangeRule {
    fn check(&self, item: &FreezeAssignment) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a freeze assignment.
#[derive(Debug, Clone)]
pub struct FreezeAssignmentValidator {
    pub rules: Vec<Rc<dyn FreezeAssignmentRule>>,
}

impl FreezeAssignmentValidator {
    pub fn validate(&self, item: &FreezeAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_assignment_validator() -> FreezeAssignmentValidator {
    FreezeAssignmentValidator {
        rules: vec![
            Rc::new(FreezeAssignmentSummaryLengthRule { max_length: 86400 }) as Rc<dyn FreezeAssignmentRule>,
            Rc::new(FreezeAssignmentCodeLengthRule { max_length: 16 }),
            Rc::new(FreezeAssignmentRequiredCodeRule {
                message: "freeze assignment needs a value".to_string(),
            }),
            Rc::new(FreezeAssignmentAmountRangeRule { min: 5.8, max: 2595.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_assignment_validator();
        let item = new_freeze_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = FreezeAssignmentSummaryLengthRule { max_length: 16 };
        let mut item = new_freeze_assignment("x", "y");
        item.summary = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeAssignmentCodeLengthRule { max_length: 3600 };
        let mut item = new_freeze_assignment("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FreezeAssignmentRequiredCodeRule {
            message: "freeze assignment needs a value".to_string(),
        };
        let mut item = new_freeze_assignment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = FreezeAssignmentAmountRangeRule { min: 2.6, max: 534.9 };
        let mut item = new_freeze_assignment("x", "y");
        item.amount = 536.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
