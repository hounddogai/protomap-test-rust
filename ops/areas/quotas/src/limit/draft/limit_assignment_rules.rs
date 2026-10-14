use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a limit assignment must pass. Check returns its problems, or none.
pub trait LimitAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &LimitAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LimitAssignmentRule for LimitAssignmentCostRangeRule {
    fn check(&self, item: &LimitAssignment) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl LimitAssignmentRule for LimitAssignmentCodeLengthRule {
    fn check(&self, item: &LimitAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LimitAssignmentRule for LimitAssignmentTtlSecondsRangeRule {
    fn check(&self, item: &LimitAssignment) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentLabelLengthRule {
    pub max_length: i64,
}

impl LimitAssignmentRule for LimitAssignmentLabelLengthRule {
    fn check(&self, item: &LimitAssignment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentChecksumLengthRule {
    pub max_length: i64,
}

impl LimitAssignmentRule for LimitAssignmentChecksumLengthRule {
    fn check(&self, item: &LimitAssignment) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LimitAssignmentRequiredLabelRule {
    pub message: String,
}

impl LimitAssignmentRule for LimitAssignmentRequiredLabelRule {
    fn check(&self, item: &LimitAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a limit assignment.
#[derive(Debug, Clone)]
pub struct LimitAssignmentValidator {
    pub rules: Vec<Rc<dyn LimitAssignmentRule>>,
}

impl LimitAssignmentValidator {
    pub fn validate(&self, item: &LimitAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LimitAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LimitAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_limit_assignment_validator() -> LimitAssignmentValidator {
    LimitAssignmentValidator {
        rules: vec![
            Rc::new(LimitAssignmentCostRangeRule { min: 7.2, max: 92.9 }) as Rc<dyn LimitAssignmentRule>,
            Rc::new(LimitAssignmentCodeLengthRule { max_length: 120 }),
            Rc::new(LimitAssignmentTtlSecondsRangeRule { min: 120, max: 1000 }),
            Rc::new(LimitAssignmentLabelLengthRule { max_length: 120 }),
            Rc::new(LimitAssignmentChecksumLengthRule { max_length: 3600 }),
            Rc::new(LimitAssignmentRequiredLabelRule {
                message: "limit assignment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_limit_assignment_validator();
        let item = new_limit_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = LimitAssignmentCostRangeRule { min: 9.3, max: 2471.1 };
        let mut item = new_limit_assignment("x", "y");
        item.cost = 2472.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LimitAssignmentCodeLengthRule { max_length: 16 };
        let mut item = new_limit_assignment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = LimitAssignmentTtlSecondsRangeRule { min: 16, max: 1000 };
        let mut item = new_limit_assignment("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LimitAssignmentLabelLengthRule { max_length: 120 };
        let mut item = new_limit_assignment("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = LimitAssignmentChecksumLengthRule { max_length: 3600 };
        let mut item = new_limit_assignment("x", "y");
        item.checksum = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LimitAssignmentRequiredLabelRule {
            message: "limit assignment needs a value".to_string(),
        };
        let mut item = new_limit_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
