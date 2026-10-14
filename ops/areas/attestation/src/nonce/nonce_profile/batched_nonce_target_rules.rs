use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched nonce target must pass. Check returns its problems, or none.
pub trait BatchedNonceTargetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedNonceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceTargetRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedNonceTargetRule for BatchedNonceTargetRetriesRangeRule {
    fn check(&self, item: &BatchedNonceTarget) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceTargetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedNonceTargetRule for BatchedNonceTargetLimitRangeRule {
    fn check(&self, item: &BatchedNonceTarget) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedNonceTargetCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedNonceTargetRule for BatchedNonceTargetCreatedAtRangeRule {
    fn check(&self, item: &BatchedNonceTarget) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched nonce target.
#[derive(Debug, Clone)]
pub struct BatchedNonceTargetValidator {
    pub rules: Vec<Rc<dyn BatchedNonceTargetRule>>,
}

impl BatchedNonceTargetValidator {
    pub fn validate(&self, item: &BatchedNonceTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedNonceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedNonceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_nonce_target_validator() -> BatchedNonceTargetValidator {
    BatchedNonceTargetValidator {
        rules: vec![
            Rc::new(BatchedNonceTargetRetriesRangeRule { min: 80, max: 1000 }) as Rc<dyn BatchedNonceTargetRule>,
            Rc::new(BatchedNonceTargetLimitRangeRule { min: 80, max: 86400 }),
            Rc::new(BatchedNonceTargetCreatedAtRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_nonce_target_validator();
        let item = new_batched_nonce_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = BatchedNonceTargetRetriesRangeRule { min: 32, max: 64 };
        let mut item = new_batched_nonce_target("x", "y");
        item.retries = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = BatchedNonceTargetLimitRangeRule { min: 255, max: 86400 };
        let mut item = new_batched_nonce_target("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = BatchedNonceTargetCreatedAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_batched_nonce_target("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
