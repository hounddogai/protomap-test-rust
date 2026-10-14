use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched policy allocation must pass. Check returns its problems, or none.
pub trait BatchedPolicyAllocationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAllocationRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPolicyAllocationRule for BatchedPolicyAllocationRevisionRangeRule {
    fn check(&self, item: &BatchedPolicyAllocation) -> Vec<String> {
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
pub struct BatchedPolicyAllocationCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPolicyAllocationRule for BatchedPolicyAllocationCapacityRangeRule {
    fn check(&self, item: &BatchedPolicyAllocation) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAllocationRequiredCodeRule {
    pub message: String,
}

impl BatchedPolicyAllocationRule for BatchedPolicyAllocationRequiredCodeRule {
    fn check(&self, item: &BatchedPolicyAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyAllocationLabelLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyAllocationRule for BatchedPolicyAllocationLabelLengthRule {
    fn check(&self, item: &BatchedPolicyAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched policy allocation.
#[derive(Debug, Clone)]
pub struct BatchedPolicyAllocationValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyAllocationRule>>,
}

impl BatchedPolicyAllocationValidator {
    pub fn validate(&self, item: &BatchedPolicyAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedPolicyAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_allocation_validator() -> BatchedPolicyAllocationValidator {
    BatchedPolicyAllocationValidator {
        rules: vec![
            Rc::new(BatchedPolicyAllocationRevisionRangeRule { min: 32, max: 80 })
                as Rc<dyn BatchedPolicyAllocationRule>,
            Rc::new(BatchedPolicyAllocationCapacityRangeRule { min: 80, max: 1000 }),
            Rc::new(BatchedPolicyAllocationRequiredCodeRule {
                message: "batched policy allocation needs a value".to_string(),
            }),
            Rc::new(BatchedPolicyAllocationLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_policy_allocation_validator();
        let item = new_batched_policy_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = BatchedPolicyAllocationRevisionRangeRule { min: 120, max: 255 };
        let mut item = new_batched_policy_allocation("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = BatchedPolicyAllocationCapacityRangeRule { min: 120, max: 86400 };
        let mut item = new_batched_policy_allocation("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedPolicyAllocationRequiredCodeRule {
            message: "batched policy allocation needs an entry".to_string(),
        };
        let mut item = new_batched_policy_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedPolicyAllocationLabelLengthRule { max_length: 64 };
        let mut item = new_batched_policy_allocation("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
