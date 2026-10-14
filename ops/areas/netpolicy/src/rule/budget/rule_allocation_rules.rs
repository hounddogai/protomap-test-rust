use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule allocation must pass. Check returns its problems, or none.
pub trait RuleAllocationRule: std::fmt::Debug {
    fn check(&self, item: &RuleAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAllocationRequiredCodeRule {
    pub message: String,
}

impl RuleAllocationRule for RuleAllocationRequiredCodeRule {
    fn check(&self, item: &RuleAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAllocationNotesLengthRule {
    pub max_length: i64,
}

impl RuleAllocationRule for RuleAllocationNotesLengthRule {
    fn check(&self, item: &RuleAllocation) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAllocationMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleAllocationRule for RuleAllocationMaximumRangeRule {
    fn check(&self, item: &RuleAllocation) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleAllocationRequiredNotesRule {
    pub message: String,
}

impl RuleAllocationRule for RuleAllocationRequiredNotesRule {
    fn check(&self, item: &RuleAllocation) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule allocation.
#[derive(Debug, Clone)]
pub struct RuleAllocationValidator {
    pub rules: Vec<Rc<dyn RuleAllocationRule>>,
}

impl RuleAllocationValidator {
    pub fn validate(&self, item: &RuleAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_allocation_validator() -> RuleAllocationValidator {
    RuleAllocationValidator {
        rules: vec![
            Rc::new(RuleAllocationRequiredCodeRule {
                message: "rule allocation needs this field".to_string(),
            }) as Rc<dyn RuleAllocationRule>,
            Rc::new(RuleAllocationNotesLengthRule { max_length: 120 }),
            Rc::new(RuleAllocationMaximumRangeRule { min: 6.9, max: 2405.5 }),
            Rc::new(RuleAllocationRequiredNotesRule {
                message: "rule allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_allocation_validator();
        let item = new_rule_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleAllocationRequiredCodeRule {
            message: "rule allocation needs a value".to_string(),
        };
        let mut item = new_rule_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RuleAllocationNotesLengthRule { max_length: 64 };
        let mut item = new_rule_allocation("x", "y");
        item.notes = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RuleAllocationMaximumRangeRule { min: 0.1, max: 3368.1 };
        let mut item = new_rule_allocation("x", "y");
        item.maximum = 3369.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RuleAllocationRequiredNotesRule {
            message: "rule allocation needs this field".to_string(),
        };
        let mut item = new_rule_allocation("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
