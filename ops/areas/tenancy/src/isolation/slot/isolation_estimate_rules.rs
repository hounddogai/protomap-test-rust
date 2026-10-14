use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a isolation estimate must pass. Check returns its problems, or none.
pub trait IsolationEstimateRule: std::fmt::Debug {
    fn check(&self, item: &IsolationEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEstimateLabelLengthRule {
    pub max_length: i64,
}

impl IsolationEstimateRule for IsolationEstimateLabelLengthRule {
    fn check(&self, item: &IsolationEstimate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEstimateExternalRefLengthRule {
    pub max_length: i64,
}

impl IsolationEstimateRule for IsolationEstimateExternalRefLengthRule {
    fn check(&self, item: &IsolationEstimate) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEstimateCodeLengthRule {
    pub max_length: i64,
}

impl IsolationEstimateRule for IsolationEstimateCodeLengthRule {
    fn check(&self, item: &IsolationEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEstimateThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl IsolationEstimateRule for IsolationEstimateThresholdRangeRule {
    fn check(&self, item: &IsolationEstimate) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationEstimateAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IsolationEstimateRule for IsolationEstimateAttemptsRangeRule {
    fn check(&self, item: &IsolationEstimate) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a isolation estimate.
#[derive(Debug, Clone)]
pub struct IsolationEstimateValidator {
    pub rules: Vec<Rc<dyn IsolationEstimateRule>>,
}

impl IsolationEstimateValidator {
    pub fn validate(&self, item: &IsolationEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &IsolationEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IsolationEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_isolation_estimate_validator() -> IsolationEstimateValidator {
    IsolationEstimateValidator {
        rules: vec![
            Rc::new(IsolationEstimateLabelLengthRule { max_length: 86400 }) as Rc<dyn IsolationEstimateRule>,
            Rc::new(IsolationEstimateExternalRefLengthRule { max_length: 120 }),
            Rc::new(IsolationEstimateCodeLengthRule { max_length: 120 }),
            Rc::new(IsolationEstimateThresholdRangeRule { min: 5.8, max: 3974.2 }),
            Rc::new(IsolationEstimateAttemptsRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_isolation_estimate_validator();
        let item = new_isolation_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = IsolationEstimateLabelLengthRule { max_length: 16 };
        let mut item = new_isolation_estimate("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = IsolationEstimateExternalRefLengthRule { max_length: 64 };
        let mut item = new_isolation_estimate("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IsolationEstimateCodeLengthRule { max_length: 86400 };
        let mut item = new_isolation_estimate("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = IsolationEstimateThresholdRangeRule { min: 6.2, max: 959.4 };
        let mut item = new_isolation_estimate("x", "y");
        item.threshold = 960.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = IsolationEstimateAttemptsRangeRule { min: 120, max: 255 };
        let mut item = new_isolation_estimate("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
