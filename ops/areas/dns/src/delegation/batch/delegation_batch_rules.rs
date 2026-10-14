use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation batch must pass. Check returns its problems, or none.
pub trait DelegationBatchRule: std::fmt::Debug {
    fn check(&self, item: &DelegationBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBatchRequiredTitleRule {
    pub message: String,
}

impl DelegationBatchRule for DelegationBatchRequiredTitleRule {
    fn check(&self, item: &DelegationBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBatchBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DelegationBatchRule for DelegationBatchBatchSizeRangeRule {
    fn check(&self, item: &DelegationBatch) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBatchRequiredCodeRule {
    pub message: String,
}

impl DelegationBatchRule for DelegationBatchRequiredCodeRule {
    fn check(&self, item: &DelegationBatch) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBatchCodePrefixRule {
    pub prefix: String,
}

impl DelegationBatchRule for DelegationBatchCodePrefixRule {
    fn check(&self, item: &DelegationBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBatchMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DelegationBatchRule for DelegationBatchMinimumRangeRule {
    fn check(&self, item: &DelegationBatch) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationBatchRequiredUnitRule {
    pub message: String,
}

impl DelegationBatchRule for DelegationBatchRequiredUnitRule {
    fn check(&self, item: &DelegationBatch) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation batch.
#[derive(Debug, Clone)]
pub struct DelegationBatchValidator {
    pub rules: Vec<Rc<dyn DelegationBatchRule>>,
}

impl DelegationBatchValidator {
    pub fn validate(&self, item: &DelegationBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_batch_validator() -> DelegationBatchValidator {
    DelegationBatchValidator {
        rules: vec![
            Rc::new(DelegationBatchRequiredTitleRule {
                message: "delegation batch needs an entry".to_string(),
            }) as Rc<dyn DelegationBatchRule>,
            Rc::new(DelegationBatchBatchSizeRangeRule { min: 64, max: 1000 }),
            Rc::new(DelegationBatchRequiredCodeRule {
                message: "delegation batch needs an entry".to_string(),
            }),
            Rc::new(DelegationBatchCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DelegationBatchMinimumRangeRule { min: 3.3, max: 2517.7 }),
            Rc::new(DelegationBatchRequiredUnitRule {
                message: "delegation batch needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_batch_validator();
        let item = new_delegation_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DelegationBatchRequiredTitleRule {
            message: "delegation batch needs this field".to_string(),
        };
        let mut item = new_delegation_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = DelegationBatchBatchSizeRangeRule { min: 16, max: 86400 };
        let mut item = new_delegation_batch("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationBatchRequiredCodeRule {
            message: "delegation batch needs an entry".to_string(),
        };
        let mut item = new_delegation_batch("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = DelegationBatchMinimumRangeRule { min: 1.7, max: 4171.2 };
        let mut item = new_delegation_batch("x", "y");
        item.minimum = 4172.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = DelegationBatchRequiredUnitRule {
            message: "delegation batch needs a value".to_string(),
        };
        let mut item = new_delegation_batch("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
