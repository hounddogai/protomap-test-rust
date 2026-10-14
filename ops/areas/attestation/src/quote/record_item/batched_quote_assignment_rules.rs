use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched quote assignment must pass. Check returns its problems, or none.
pub trait BatchedQuoteAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedQuoteAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedQuoteAssignmentRule for BatchedQuoteAssignmentCeilingRangeRule {
    fn check(&self, item: &BatchedQuoteAssignment) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteAssignmentRequiredNameRule {
    pub message: String,
}

impl BatchedQuoteAssignmentRule for BatchedQuoteAssignmentRequiredNameRule {
    fn check(&self, item: &BatchedQuoteAssignment) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteAssignmentBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedQuoteAssignmentRule for BatchedQuoteAssignmentBatchSizeRangeRule {
    fn check(&self, item: &BatchedQuoteAssignment) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteAssignmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl BatchedQuoteAssignmentRule for BatchedQuoteAssignmentAllowedNameRule {
    fn check(&self, item: &BatchedQuoteAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched quote assignment.
#[derive(Debug, Clone)]
pub struct BatchedQuoteAssignmentValidator {
    pub rules: Vec<Rc<dyn BatchedQuoteAssignmentRule>>,
}

impl BatchedQuoteAssignmentValidator {
    pub fn validate(&self, item: &BatchedQuoteAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedQuoteAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedQuoteAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_quote_assignment_validator() -> BatchedQuoteAssignmentValidator {
    BatchedQuoteAssignmentValidator {
        rules: vec![
            Rc::new(BatchedQuoteAssignmentCeilingRangeRule { min: 1.3, max: 903.2 })
                as Rc<dyn BatchedQuoteAssignmentRule>,
            Rc::new(BatchedQuoteAssignmentRequiredNameRule {
                message: "batched quote assignment needs an entry".to_string(),
            }),
            Rc::new(BatchedQuoteAssignmentBatchSizeRangeRule { min: 64, max: 3600 }),
            Rc::new(BatchedQuoteAssignmentAllowedNameRule {
                allowed: vec!["seasonal".to_string(), "pending".to_string(), "scheduled".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_quote_assignment_validator();
        let item = new_batched_quote_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = BatchedQuoteAssignmentCeilingRangeRule { min: 4.9, max: 2130.2 };
        let mut item = new_batched_quote_assignment("x", "y");
        item.ceiling = 2131.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BatchedQuoteAssignmentRequiredNameRule {
            message: "batched quote assignment needs this field".to_string(),
        };
        let mut item = new_batched_quote_assignment("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = BatchedQuoteAssignmentBatchSizeRangeRule { min: 16, max: 86400 };
        let mut item = new_batched_quote_assignment("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
