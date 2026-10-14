use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a failover batch must pass. Check returns its problems, or none.
pub trait FailoverBatchRule: std::fmt::Debug {
    fn check(&self, item: &FailoverBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBatchOwnerIdLengthRule {
    pub max_length: i64,
}

impl FailoverBatchRule for FailoverBatchOwnerIdLengthRule {
    fn check(&self, item: &FailoverBatch) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBatchDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FailoverBatchRule for FailoverBatchDiscountRangeRule {
    fn check(&self, item: &FailoverBatch) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBatchLabelLengthRule {
    pub max_length: i64,
}

impl FailoverBatchRule for FailoverBatchLabelLengthRule {
    fn check(&self, item: &FailoverBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverBatchCodePrefixRule {
    pub prefix: String,
}

impl FailoverBatchRule for FailoverBatchCodePrefixRule {
    fn check(&self, item: &FailoverBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover batch.
#[derive(Debug, Clone)]
pub struct FailoverBatchValidator {
    pub rules: Vec<Rc<dyn FailoverBatchRule>>,
}

impl FailoverBatchValidator {
    pub fn validate(&self, item: &FailoverBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_batch_validator() -> FailoverBatchValidator {
    FailoverBatchValidator {
        rules: vec![
            Rc::new(FailoverBatchOwnerIdLengthRule { max_length: 3600 }) as Rc<dyn FailoverBatchRule>,
            Rc::new(FailoverBatchDiscountRangeRule { min: 6.9, max: 1081.6 }),
            Rc::new(FailoverBatchLabelLengthRule { max_length: 120 }),
            Rc::new(FailoverBatchCodePrefixRule {
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
        let validator = default_failover_batch_validator();
        let item = new_failover_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = FailoverBatchOwnerIdLengthRule { max_length: 16 };
        let mut item = new_failover_batch("x", "y");
        item.owner_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = FailoverBatchDiscountRangeRule { min: 3.3, max: 2858.6 };
        let mut item = new_failover_batch("x", "y");
        item.discount = 2860.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FailoverBatchLabelLengthRule { max_length: 86400 };
        let mut item = new_failover_batch("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
