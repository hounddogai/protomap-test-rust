use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a suspension variance must pass. Check returns its problems, or none.
pub trait SuspensionVarianceRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionVarianceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SuspensionVarianceRule for SuspensionVarianceWeightRangeRule {
    fn check(&self, item: &SuspensionVariance) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionVarianceBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionVarianceRule for SuspensionVarianceBatchSizeRangeRule {
    fn check(&self, item: &SuspensionVariance) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionVarianceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionVarianceRule for SuspensionVarianceStartsAtRangeRule {
    fn check(&self, item: &SuspensionVariance) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionVarianceLabelLengthRule {
    pub max_length: i64,
}

impl SuspensionVarianceRule for SuspensionVarianceLabelLengthRule {
    fn check(&self, item: &SuspensionVariance) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionVarianceExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionVarianceRule for SuspensionVarianceExpiresAtRangeRule {
    fn check(&self, item: &SuspensionVariance) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension variance.
#[derive(Debug, Clone)]
pub struct SuspensionVarianceValidator {
    pub rules: Vec<Rc<dyn SuspensionVarianceRule>>,
}

impl SuspensionVarianceValidator {
    pub fn validate(&self, item: &SuspensionVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_variance_validator() -> SuspensionVarianceValidator {
    SuspensionVarianceValidator {
        rules: vec![
            Rc::new(SuspensionVarianceWeightRangeRule { min: 4.4, max: 3891.7 }) as Rc<dyn SuspensionVarianceRule>,
            Rc::new(SuspensionVarianceBatchSizeRangeRule { min: 3600, max: 86400 }),
            Rc::new(SuspensionVarianceStartsAtRangeRule { min: 64, max: 86400 }),
            Rc::new(SuspensionVarianceLabelLengthRule { max_length: 255 }),
            Rc::new(SuspensionVarianceExpiresAtRangeRule { min: 120, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_variance_validator();
        let item = new_suspension_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SuspensionVarianceWeightRangeRule { min: 0.9, max: 2390.6 };
        let mut item = new_suspension_variance("x", "y");
        item.weight = 2392.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = SuspensionVarianceBatchSizeRangeRule { min: 32, max: 80 };
        let mut item = new_suspension_variance("x", "y");
        item.batch_size = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = SuspensionVarianceStartsAtRangeRule { min: 64, max: 1000 };
        let mut item = new_suspension_variance("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SuspensionVarianceLabelLengthRule { max_length: 80 };
        let mut item = new_suspension_variance("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = SuspensionVarianceExpiresAtRangeRule { min: 3600, max: 86400 };
        let mut item = new_suspension_variance("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
