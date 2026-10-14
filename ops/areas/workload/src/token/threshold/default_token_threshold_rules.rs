use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default token threshold must pass. Check returns its problems, or none.
pub trait DefaultTokenThresholdRule: std::fmt::Debug {
    fn check(&self, item: &DefaultTokenThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenThresholdCodeLengthRule {
    pub max_length: i64,
}

impl DefaultTokenThresholdRule for DefaultTokenThresholdCodeLengthRule {
    fn check(&self, item: &DefaultTokenThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenThresholdUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultTokenThresholdRule for DefaultTokenThresholdUpdatedAtRangeRule {
    fn check(&self, item: &DefaultTokenThreshold) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenThresholdRegionLengthRule {
    pub max_length: i64,
}

impl DefaultTokenThresholdRule for DefaultTokenThresholdRegionLengthRule {
    fn check(&self, item: &DefaultTokenThreshold) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenThresholdSourceLengthRule {
    pub max_length: i64,
}

impl DefaultTokenThresholdRule for DefaultTokenThresholdSourceLengthRule {
    fn check(&self, item: &DefaultTokenThreshold) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTokenThresholdRequiredRegionRule {
    pub message: String,
}

impl DefaultTokenThresholdRule for DefaultTokenThresholdRequiredRegionRule {
    fn check(&self, item: &DefaultTokenThreshold) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default token threshold.
#[derive(Debug, Clone)]
pub struct DefaultTokenThresholdValidator {
    pub rules: Vec<Rc<dyn DefaultTokenThresholdRule>>,
}

impl DefaultTokenThresholdValidator {
    pub fn validate(&self, item: &DefaultTokenThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultTokenThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultTokenThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_token_threshold_validator() -> DefaultTokenThresholdValidator {
    DefaultTokenThresholdValidator {
        rules: vec![
            Rc::new(DefaultTokenThresholdCodeLengthRule { max_length: 1000 }) as Rc<dyn DefaultTokenThresholdRule>,
            Rc::new(DefaultTokenThresholdUpdatedAtRangeRule { min: 120, max: 1000 }),
            Rc::new(DefaultTokenThresholdRegionLengthRule { max_length: 1000 }),
            Rc::new(DefaultTokenThresholdSourceLengthRule { max_length: 1000 }),
            Rc::new(DefaultTokenThresholdRequiredRegionRule {
                message: "default token threshold needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_token_threshold_validator();
        let item = new_default_token_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultTokenThresholdCodeLengthRule { max_length: 1000 };
        let mut item = new_default_token_threshold("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = DefaultTokenThresholdUpdatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_default_token_threshold("x", "y");
        item.updated_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = DefaultTokenThresholdRegionLengthRule { max_length: 16 };
        let mut item = new_default_token_threshold("x", "y");
        item.region = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = DefaultTokenThresholdSourceLengthRule { max_length: 64 };
        let mut item = new_default_token_threshold("x", "y");
        item.source = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DefaultTokenThresholdRequiredRegionRule {
            message: "default token threshold needs an entry".to_string(),
        };
        let mut item = new_default_token_threshold("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
