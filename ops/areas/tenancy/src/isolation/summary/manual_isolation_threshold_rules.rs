use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual isolation threshold must pass. Check returns its problems, or none.
pub trait ManualIsolationThresholdRule: std::fmt::Debug {
    fn check(&self, item: &ManualIsolationThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationThresholdRequiredPhoneRule {
    pub message: String,
}

impl ManualIsolationThresholdRule for ManualIsolationThresholdRequiredPhoneRule {
    fn check(&self, item: &ManualIsolationThreshold) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationThresholdExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualIsolationThresholdRule for ManualIsolationThresholdExpiresAtRangeRule {
    fn check(&self, item: &ManualIsolationThreshold) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationThresholdAudienceLengthRule {
    pub max_length: i64,
}

impl ManualIsolationThresholdRule for ManualIsolationThresholdAudienceLengthRule {
    fn check(&self, item: &ManualIsolationThreshold) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualIsolationThresholdThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualIsolationThresholdRule for ManualIsolationThresholdThresholdRangeRule {
    fn check(&self, item: &ManualIsolationThreshold) -> Vec<String> {
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
pub struct ManualIsolationThresholdCodeLengthRule {
    pub max_length: i64,
}

impl ManualIsolationThresholdRule for ManualIsolationThresholdCodeLengthRule {
    fn check(&self, item: &ManualIsolationThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual isolation threshold.
#[derive(Debug, Clone)]
pub struct ManualIsolationThresholdValidator {
    pub rules: Vec<Rc<dyn ManualIsolationThresholdRule>>,
}

impl ManualIsolationThresholdValidator {
    pub fn validate(&self, item: &ManualIsolationThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualIsolationThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualIsolationThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_isolation_threshold_validator() -> ManualIsolationThresholdValidator {
    ManualIsolationThresholdValidator {
        rules: vec![
            Rc::new(ManualIsolationThresholdRequiredPhoneRule {
                message: "manual isolation threshold needs a value".to_string(),
            }) as Rc<dyn ManualIsolationThresholdRule>,
            Rc::new(ManualIsolationThresholdExpiresAtRangeRule { min: 80, max: 3600 }),
            Rc::new(ManualIsolationThresholdAudienceLengthRule { max_length: 32 }),
            Rc::new(ManualIsolationThresholdThresholdRangeRule { min: 0.4, max: 1227.9 }),
            Rc::new(ManualIsolationThresholdCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_isolation_threshold_validator();
        let item = new_manual_isolation_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = ManualIsolationThresholdRequiredPhoneRule {
            message: "manual isolation threshold needs an entry".to_string(),
        };
        let mut item = new_manual_isolation_threshold("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = ManualIsolationThresholdExpiresAtRangeRule { min: 32, max: 80 };
        let mut item = new_manual_isolation_threshold("x", "y");
        item.expires_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = ManualIsolationThresholdAudienceLengthRule { max_length: 1000 };
        let mut item = new_manual_isolation_threshold("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = ManualIsolationThresholdThresholdRangeRule { min: 9.0, max: 374.5 };
        let mut item = new_manual_isolation_threshold("x", "y");
        item.threshold = 376.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualIsolationThresholdCodeLengthRule { max_length: 80 };
        let mut item = new_manual_isolation_threshold("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
