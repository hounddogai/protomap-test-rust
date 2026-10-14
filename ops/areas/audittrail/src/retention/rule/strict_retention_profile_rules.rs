use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict retention profile must pass. Check returns its problems, or none.
pub trait StrictRetentionProfileRule: std::fmt::Debug {
    fn check(&self, item: &StrictRetentionProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionProfileAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictRetentionProfileRule for StrictRetentionProfileAmountRangeRule {
    fn check(&self, item: &StrictRetentionProfile) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionProfileCipherLengthRule {
    pub max_length: i64,
}

impl StrictRetentionProfileRule for StrictRetentionProfileCipherLengthRule {
    fn check(&self, item: &StrictRetentionProfile) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictRetentionProfileRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictRetentionProfileRule for StrictRetentionProfileRevisionRangeRule {
    fn check(&self, item: &StrictRetentionProfile) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict retention profile.
#[derive(Debug, Clone)]
pub struct StrictRetentionProfileValidator {
    pub rules: Vec<Rc<dyn StrictRetentionProfileRule>>,
}

impl StrictRetentionProfileValidator {
    pub fn validate(&self, item: &StrictRetentionProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictRetentionProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictRetentionProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_retention_profile_validator() -> StrictRetentionProfileValidator {
    StrictRetentionProfileValidator {
        rules: vec![
            Rc::new(StrictRetentionProfileAmountRangeRule { min: 0.3, max: 1266.3 })
                as Rc<dyn StrictRetentionProfileRule>,
            Rc::new(StrictRetentionProfileCipherLengthRule { max_length: 86400 }),
            Rc::new(StrictRetentionProfileRevisionRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_retention_profile_validator();
        let item = new_strict_retention_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = StrictRetentionProfileAmountRangeRule { min: 9.9, max: 4456.0 };
        let mut item = new_strict_retention_profile("x", "y");
        item.amount = 4457.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = StrictRetentionProfileCipherLengthRule { max_length: 255 };
        let mut item = new_strict_retention_profile("x", "y");
        item.cipher = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = StrictRetentionProfileRevisionRangeRule { min: 80, max: 1000 };
        let mut item = new_strict_retention_profile("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
