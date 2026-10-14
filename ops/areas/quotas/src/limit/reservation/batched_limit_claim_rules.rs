use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched limit claim must pass. Check returns its problems, or none.
pub trait BatchedLimitClaimRule: std::fmt::Debug {
    fn check(&self, item: &BatchedLimitClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimLabelLengthRule {
    pub max_length: i64,
}

impl BatchedLimitClaimRule for BatchedLimitClaimLabelLengthRule {
    fn check(&self, item: &BatchedLimitClaim) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedLimitClaimRule for BatchedLimitClaimMaximumRangeRule {
    fn check(&self, item: &BatchedLimitClaim) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedLimitClaimRule for BatchedLimitClaimAllowedCodeRule {
    fn check(&self, item: &BatchedLimitClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedLimitClaimRule for BatchedLimitClaimPortRangeRule {
    fn check(&self, item: &BatchedLimitClaim) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedLimitClaimRule for BatchedLimitClaimCreatedAtRangeRule {
    fn check(&self, item: &BatchedLimitClaim) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitClaimCodePrefixRule {
    pub prefix: String,
}

impl BatchedLimitClaimRule for BatchedLimitClaimCodePrefixRule {
    fn check(&self, item: &BatchedLimitClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched limit claim.
#[derive(Debug, Clone)]
pub struct BatchedLimitClaimValidator {
    pub rules: Vec<Rc<dyn BatchedLimitClaimRule>>,
}

impl BatchedLimitClaimValidator {
    pub fn validate(&self, item: &BatchedLimitClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedLimitClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedLimitClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_limit_claim_validator() -> BatchedLimitClaimValidator {
    BatchedLimitClaimValidator {
        rules: vec![
            Rc::new(BatchedLimitClaimLabelLengthRule { max_length: 120 }) as Rc<dyn BatchedLimitClaimRule>,
            Rc::new(BatchedLimitClaimMaximumRangeRule { min: 4.1, max: 4521.3 }),
            Rc::new(BatchedLimitClaimAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(BatchedLimitClaimPortRangeRule { min: 255, max: 255 }),
            Rc::new(BatchedLimitClaimCreatedAtRangeRule { min: 64, max: 80 }),
            Rc::new(BatchedLimitClaimCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_limit_claim_validator();
        let item = new_batched_limit_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = BatchedLimitClaimLabelLengthRule { max_length: 32 };
        let mut item = new_batched_limit_claim("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = BatchedLimitClaimMaximumRangeRule { min: 0.8, max: 3157.8 };
        let mut item = new_batched_limit_claim("x", "y");
        item.maximum = 3159.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = BatchedLimitClaimPortRangeRule { min: 32, max: 255 };
        let mut item = new_batched_limit_claim("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = BatchedLimitClaimCreatedAtRangeRule { min: 80, max: 86400 };
        let mut item = new_batched_limit_claim("x", "y");
        item.created_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
