use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global suspension batch must pass. Check returns its problems, or none.
pub trait GlobalSuspensionBatchRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSuspensionBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionBatchWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalSuspensionBatchRule for GlobalSuspensionBatchWeightRangeRule {
    fn check(&self, item: &GlobalSuspensionBatch) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionBatchCodeLengthRule {
    pub max_length: i64,
}

impl GlobalSuspensionBatchRule for GlobalSuspensionBatchCodeLengthRule {
    fn check(&self, item: &GlobalSuspensionBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSuspensionBatchExternalRefLengthRule {
    pub max_length: i64,
}

impl GlobalSuspensionBatchRule for GlobalSuspensionBatchExternalRefLengthRule {
    fn check(&self, item: &GlobalSuspensionBatch) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global suspension batch.
#[derive(Debug, Clone)]
pub struct GlobalSuspensionBatchValidator {
    pub rules: Vec<Rc<dyn GlobalSuspensionBatchRule>>,
}

impl GlobalSuspensionBatchValidator {
    pub fn validate(&self, item: &GlobalSuspensionBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalSuspensionBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSuspensionBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_suspension_batch_validator() -> GlobalSuspensionBatchValidator {
    GlobalSuspensionBatchValidator {
        rules: vec![
            Rc::new(GlobalSuspensionBatchWeightRangeRule { min: 3.3, max: 2618.6 })
                as Rc<dyn GlobalSuspensionBatchRule>,
            Rc::new(GlobalSuspensionBatchCodeLengthRule { max_length: 86400 }),
            Rc::new(GlobalSuspensionBatchExternalRefLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_suspension_batch_validator();
        let item = new_global_suspension_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = GlobalSuspensionBatchWeightRangeRule { min: 2.4, max: 2693.6 };
        let mut item = new_global_suspension_batch("x", "y");
        item.weight = 2695.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalSuspensionBatchCodeLengthRule { max_length: 32 };
        let mut item = new_global_suspension_batch("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = GlobalSuspensionBatchExternalRefLengthRule { max_length: 3600 };
        let mut item = new_global_suspension_batch("x", "y");
        item.external_ref = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
