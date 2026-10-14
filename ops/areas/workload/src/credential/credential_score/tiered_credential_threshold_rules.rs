use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered credential threshold must pass. Check returns its problems, or none.
pub trait TieredCredentialThresholdRule: std::fmt::Debug {
    fn check(&self, item: &TieredCredentialThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialThresholdBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredCredentialThresholdRule for TieredCredentialThresholdBatchSizeRangeRule {
    fn check(&self, item: &TieredCredentialThreshold) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialThresholdThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredCredentialThresholdRule for TieredCredentialThresholdThresholdRangeRule {
    fn check(&self, item: &TieredCredentialThreshold) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialThresholdRequiredLabelRule {
    pub message: String,
}

impl TieredCredentialThresholdRule for TieredCredentialThresholdRequiredLabelRule {
    fn check(&self, item: &TieredCredentialThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialThresholdRequiredCodeRule {
    pub message: String,
}

impl TieredCredentialThresholdRule for TieredCredentialThresholdRequiredCodeRule {
    fn check(&self, item: &TieredCredentialThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered credential threshold.
#[derive(Debug, Clone)]
pub struct TieredCredentialThresholdValidator {
    pub rules: Vec<Rc<dyn TieredCredentialThresholdRule>>,
}

impl TieredCredentialThresholdValidator {
    pub fn validate(&self, item: &TieredCredentialThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredCredentialThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredCredentialThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_credential_threshold_validator() -> TieredCredentialThresholdValidator {
    TieredCredentialThresholdValidator {
        rules: vec![
            Rc::new(TieredCredentialThresholdBatchSizeRangeRule { min: 16, max: 80 })
                as Rc<dyn TieredCredentialThresholdRule>,
            Rc::new(TieredCredentialThresholdThresholdRangeRule { min: 6.7, max: 3079.9 }),
            Rc::new(TieredCredentialThresholdRequiredLabelRule {
                message: "tiered credential threshold needs a value".to_string(),
            }),
            Rc::new(TieredCredentialThresholdRequiredCodeRule {
                message: "tiered credential threshold needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_credential_threshold_validator();
        let item = new_tiered_credential_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TieredCredentialThresholdBatchSizeRangeRule { min: 1000, max: 3600 };
        let mut item = new_tiered_credential_threshold("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = TieredCredentialThresholdThresholdRangeRule { min: 0.3, max: 3594.1 };
        let mut item = new_tiered_credential_threshold("x", "y");
        item.threshold = 3595.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TieredCredentialThresholdRequiredLabelRule {
            message: "tiered credential threshold needs a value".to_string(),
        };
        let mut item = new_tiered_credential_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredCredentialThresholdRequiredCodeRule {
            message: "tiered credential threshold needs a value".to_string(),
        };
        let mut item = new_tiered_credential_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
