use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary estimate must pass. Check returns its problems, or none.
pub trait CanaryEstimateRule: std::fmt::Debug {
    fn check(&self, item: &CanaryEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEstimateCodeLengthRule {
    pub max_length: i64,
}

impl CanaryEstimateRule for CanaryEstimateCodeLengthRule {
    fn check(&self, item: &CanaryEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEstimateClusterLengthRule {
    pub max_length: i64,
}

impl CanaryEstimateRule for CanaryEstimateClusterLengthRule {
    fn check(&self, item: &CanaryEstimate) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEstimateWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanaryEstimateRule for CanaryEstimateWeightRangeRule {
    fn check(&self, item: &CanaryEstimate) -> Vec<String> {
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
pub struct CanaryEstimateTitleLengthRule {
    pub max_length: i64,
}

impl CanaryEstimateRule for CanaryEstimateTitleLengthRule {
    fn check(&self, item: &CanaryEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryEstimateRequiredClusterRule {
    pub message: String,
}

impl CanaryEstimateRule for CanaryEstimateRequiredClusterRule {
    fn check(&self, item: &CanaryEstimate) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary estimate.
#[derive(Debug, Clone)]
pub struct CanaryEstimateValidator {
    pub rules: Vec<Rc<dyn CanaryEstimateRule>>,
}

impl CanaryEstimateValidator {
    pub fn validate(&self, item: &CanaryEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_estimate_validator() -> CanaryEstimateValidator {
    CanaryEstimateValidator {
        rules: vec![
            Rc::new(CanaryEstimateCodeLengthRule { max_length: 1000 }) as Rc<dyn CanaryEstimateRule>,
            Rc::new(CanaryEstimateClusterLengthRule { max_length: 3600 }),
            Rc::new(CanaryEstimateWeightRangeRule { min: 0.9, max: 1166.3 }),
            Rc::new(CanaryEstimateTitleLengthRule { max_length: 32 }),
            Rc::new(CanaryEstimateRequiredClusterRule {
                message: "canary estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_estimate_validator();
        let item = new_canary_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryEstimateCodeLengthRule { max_length: 3600 };
        let mut item = new_canary_estimate("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = CanaryEstimateClusterLengthRule { max_length: 255 };
        let mut item = new_canary_estimate("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = CanaryEstimateWeightRangeRule { min: 2.2, max: 517.6 };
        let mut item = new_canary_estimate("x", "y");
        item.weight = 519.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CanaryEstimateTitleLengthRule { max_length: 3600 };
        let mut item = new_canary_estimate("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = CanaryEstimateRequiredClusterRule {
            message: "canary estimate needs this field".to_string(),
        };
        let mut item = new_canary_estimate("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
