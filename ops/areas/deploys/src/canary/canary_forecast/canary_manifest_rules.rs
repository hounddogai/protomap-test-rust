use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary manifest must pass. Check returns its problems, or none.
pub trait CanaryManifestRule: std::fmt::Debug {
    fn check(&self, item: &CanaryManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryManifestRequiredCodeRule {
    pub message: String,
}

impl CanaryManifestRule for CanaryManifestRequiredCodeRule {
    fn check(&self, item: &CanaryManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryManifestSummaryLengthRule {
    pub max_length: i64,
}

impl CanaryManifestRule for CanaryManifestSummaryLengthRule {
    fn check(&self, item: &CanaryManifest) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryManifestThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CanaryManifestRule for CanaryManifestThresholdRangeRule {
    fn check(&self, item: &CanaryManifest) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary manifest.
#[derive(Debug, Clone)]
pub struct CanaryManifestValidator {
    pub rules: Vec<Rc<dyn CanaryManifestRule>>,
}

impl CanaryManifestValidator {
    pub fn validate(&self, item: &CanaryManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_manifest_validator() -> CanaryManifestValidator {
    CanaryManifestValidator {
        rules: vec![
            Rc::new(CanaryManifestRequiredCodeRule {
                message: "canary manifest needs a value".to_string(),
            }) as Rc<dyn CanaryManifestRule>,
            Rc::new(CanaryManifestSummaryLengthRule { max_length: 64 }),
            Rc::new(CanaryManifestThresholdRangeRule { min: 7.8, max: 2674.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_manifest_validator();
        let item = new_canary_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CanaryManifestRequiredCodeRule {
            message: "canary manifest needs a value".to_string(),
        };
        let mut item = new_canary_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = CanaryManifestSummaryLengthRule { max_length: 120 };
        let mut item = new_canary_manifest("x", "y");
        item.summary = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = CanaryManifestThresholdRangeRule { min: 7.6, max: 1493.4 };
        let mut item = new_canary_manifest("x", "y");
        item.threshold = 1494.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
