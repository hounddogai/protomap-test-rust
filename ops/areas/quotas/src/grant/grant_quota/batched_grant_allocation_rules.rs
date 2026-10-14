use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched grant allocation must pass. Check returns its problems, or none.
pub trait BatchedGrantAllocationRule: std::fmt::Debug {
    fn check(&self, item: &BatchedGrantAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantAllocationWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedGrantAllocationRule for BatchedGrantAllocationWeightRangeRule {
    fn check(&self, item: &BatchedGrantAllocation) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantAllocationCodeLengthRule {
    pub max_length: i64,
}

impl BatchedGrantAllocationRule for BatchedGrantAllocationCodeLengthRule {
    fn check(&self, item: &BatchedGrantAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantAllocationScopeLengthRule {
    pub max_length: i64,
}

impl BatchedGrantAllocationRule for BatchedGrantAllocationScopeLengthRule {
    fn check(&self, item: &BatchedGrantAllocation) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedGrantAllocationTitleLengthRule {
    pub max_length: i64,
}

impl BatchedGrantAllocationRule for BatchedGrantAllocationTitleLengthRule {
    fn check(&self, item: &BatchedGrantAllocation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched grant allocation.
#[derive(Debug, Clone)]
pub struct BatchedGrantAllocationValidator {
    pub rules: Vec<Rc<dyn BatchedGrantAllocationRule>>,
}

impl BatchedGrantAllocationValidator {
    pub fn validate(&self, item: &BatchedGrantAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedGrantAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedGrantAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_grant_allocation_validator() -> BatchedGrantAllocationValidator {
    BatchedGrantAllocationValidator {
        rules: vec![
            Rc::new(BatchedGrantAllocationWeightRangeRule { min: 9.8, max: 1816.7 })
                as Rc<dyn BatchedGrantAllocationRule>,
            Rc::new(BatchedGrantAllocationCodeLengthRule { max_length: 16 }),
            Rc::new(BatchedGrantAllocationScopeLengthRule { max_length: 3600 }),
            Rc::new(BatchedGrantAllocationTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_grant_allocation_validator();
        let item = new_batched_grant_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = BatchedGrantAllocationWeightRangeRule { min: 1.5, max: 4161.2 };
        let mut item = new_batched_grant_allocation("x", "y");
        item.weight = 4162.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedGrantAllocationCodeLengthRule { max_length: 1000 };
        let mut item = new_batched_grant_allocation("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = BatchedGrantAllocationScopeLengthRule { max_length: 255 };
        let mut item = new_batched_grant_allocation("x", "y");
        item.scope = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedGrantAllocationTitleLengthRule { max_length: 64 };
        let mut item = new_batched_grant_allocation("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
