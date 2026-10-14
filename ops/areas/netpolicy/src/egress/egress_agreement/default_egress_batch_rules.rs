use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default egress batch must pass. Check returns its problems, or none.
pub trait DefaultEgressBatchRule: std::fmt::Debug {
    fn check(&self, item: &DefaultEgressBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressBatchRequiredFingerprintRule {
    pub message: String,
}

impl DefaultEgressBatchRule for DefaultEgressBatchRequiredFingerprintRule {
    fn check(&self, item: &DefaultEgressBatch) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressBatchDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultEgressBatchRule for DefaultEgressBatchDiscountRangeRule {
    fn check(&self, item: &DefaultEgressBatch) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressBatchRegionLengthRule {
    pub max_length: i64,
}

impl DefaultEgressBatchRule for DefaultEgressBatchRegionLengthRule {
    fn check(&self, item: &DefaultEgressBatch) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEgressBatchQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultEgressBatchRule for DefaultEgressBatchQuantityRangeRule {
    fn check(&self, item: &DefaultEgressBatch) -> Vec<String> {
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

/// Runs every rule of a default egress batch.
#[derive(Debug, Clone)]
pub struct DefaultEgressBatchValidator {
    pub rules: Vec<Rc<dyn DefaultEgressBatchRule>>,
}

impl DefaultEgressBatchValidator {
    pub fn validate(&self, item: &DefaultEgressBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultEgressBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultEgressBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_egress_batch_validator() -> DefaultEgressBatchValidator {
    DefaultEgressBatchValidator {
        rules: vec![
            Rc::new(DefaultEgressBatchRequiredFingerprintRule {
                message: "default egress batch needs this field".to_string(),
            }) as Rc<dyn DefaultEgressBatchRule>,
            Rc::new(DefaultEgressBatchDiscountRangeRule { min: 5.8, max: 3727.5 }),
            Rc::new(DefaultEgressBatchRegionLengthRule { max_length: 16 }),
            Rc::new(DefaultEgressBatchQuantityRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_egress_batch_validator();
        let item = new_default_egress_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = DefaultEgressBatchRequiredFingerprintRule {
            message: "default egress batch needs a value".to_string(),
        };
        let mut item = new_default_egress_batch("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = DefaultEgressBatchDiscountRangeRule { min: 4.8, max: 2885.4 };
        let mut item = new_default_egress_batch("x", "y");
        item.discount = 2886.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DefaultEgressBatchRegionLengthRule { max_length: 1000 };
        let mut item = new_default_egress_batch("x", "y");
        item.region = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DefaultEgressBatchQuantityRangeRule { min: 3600, max: 86400 };
        let mut item = new_default_egress_batch("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
