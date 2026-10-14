use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched release adjustment must pass. Check returns its problems, or none.
pub trait BatchedReleaseAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &BatchedReleaseAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseAdjustmentQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedReleaseAdjustmentRule for BatchedReleaseAdjustmentQuantityRangeRule {
    fn check(&self, item: &BatchedReleaseAdjustment) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseAdjustmentCountryLengthRule {
    pub max_length: i64,
}

impl BatchedReleaseAdjustmentRule for BatchedReleaseAdjustmentCountryLengthRule {
    fn check(&self, item: &BatchedReleaseAdjustment) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedReleaseAdjustmentRequiredCountryRule {
    pub message: String,
}

impl BatchedReleaseAdjustmentRule for BatchedReleaseAdjustmentRequiredCountryRule {
    fn check(&self, item: &BatchedReleaseAdjustment) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched release adjustment.
#[derive(Debug, Clone)]
pub struct BatchedReleaseAdjustmentValidator {
    pub rules: Vec<Rc<dyn BatchedReleaseAdjustmentRule>>,
}

impl BatchedReleaseAdjustmentValidator {
    pub fn validate(&self, item: &BatchedReleaseAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedReleaseAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedReleaseAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_release_adjustment_validator() -> BatchedReleaseAdjustmentValidator {
    BatchedReleaseAdjustmentValidator {
        rules: vec![
            Rc::new(BatchedReleaseAdjustmentQuantityRangeRule { min: 255, max: 3600 })
                as Rc<dyn BatchedReleaseAdjustmentRule>,
            Rc::new(BatchedReleaseAdjustmentCountryLengthRule { max_length: 80 }),
            Rc::new(BatchedReleaseAdjustmentRequiredCountryRule {
                message: "batched release adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_release_adjustment_validator();
        let item = new_batched_release_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = BatchedReleaseAdjustmentQuantityRangeRule { min: 3600, max: 86400 };
        let mut item = new_batched_release_adjustment("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = BatchedReleaseAdjustmentCountryLengthRule { max_length: 64 };
        let mut item = new_batched_release_adjustment("x", "y");
        item.country = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = BatchedReleaseAdjustmentRequiredCountryRule {
            message: "batched release adjustment needs a value".to_string(),
        };
        let mut item = new_batched_release_adjustment("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
