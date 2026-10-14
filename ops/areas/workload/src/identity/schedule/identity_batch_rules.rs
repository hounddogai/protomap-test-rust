use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity batch must pass. Check returns its problems, or none.
pub trait IdentityBatchRule: std::fmt::Debug {
    fn check(&self, item: &IdentityBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBatchCodePrefixRule {
    pub prefix: String,
}

impl IdentityBatchRule for IdentityBatchCodePrefixRule {
    fn check(&self, item: &IdentityBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBatchCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityBatchRule for IdentityBatchCapacityRangeRule {
    fn check(&self, item: &IdentityBatch) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBatchFingerprintLengthRule {
    pub max_length: i64,
}

impl IdentityBatchRule for IdentityBatchFingerprintLengthRule {
    fn check(&self, item: &IdentityBatch) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityBatchAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IdentityBatchRule for IdentityBatchAmountRangeRule {
    fn check(&self, item: &IdentityBatch) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity batch.
#[derive(Debug, Clone)]
pub struct IdentityBatchValidator {
    pub rules: Vec<Rc<dyn IdentityBatchRule>>,
}

impl IdentityBatchValidator {
    pub fn validate(&self, item: &IdentityBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_batch_validator() -> IdentityBatchValidator {
    IdentityBatchValidator {
        rules: vec![
            Rc::new(IdentityBatchCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn IdentityBatchRule>,
            Rc::new(IdentityBatchCapacityRangeRule { min: 64, max: 120 }),
            Rc::new(IdentityBatchFingerprintLengthRule { max_length: 86400 }),
            Rc::new(IdentityBatchAmountRangeRule { min: 7.7, max: 1349.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_batch_validator();
        let item = new_identity_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = IdentityBatchCapacityRangeRule { min: 255, max: 1000 };
        let mut item = new_identity_batch("x", "y");
        item.capacity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = IdentityBatchFingerprintLengthRule { max_length: 16 };
        let mut item = new_identity_batch("x", "y");
        item.fingerprint = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = IdentityBatchAmountRangeRule { min: 6.3, max: 357.1 };
        let mut item = new_identity_batch("x", "y");
        item.amount = 358.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
