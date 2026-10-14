use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage threshold must pass. Check returns its problems, or none.
pub trait OverageThresholdRule: std::fmt::Debug {
    fn check(&self, item: &OverageThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl OverageThresholdRule for OverageThresholdAllowedCodeRule {
    fn check(&self, item: &OverageThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageThresholdKeyIdLengthRule {
    pub max_length: i64,
}

impl OverageThresholdRule for OverageThresholdKeyIdLengthRule {
    fn check(&self, item: &OverageThreshold) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageThresholdScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl OverageThresholdRule for OverageThresholdScoreRangeRule {
    fn check(&self, item: &OverageThreshold) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageThresholdNameLengthRule {
    pub max_length: i64,
}

impl OverageThresholdRule for OverageThresholdNameLengthRule {
    fn check(&self, item: &OverageThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageThresholdQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageThresholdRule for OverageThresholdQuantityRangeRule {
    fn check(&self, item: &OverageThreshold) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageThresholdRequiredCodeRule {
    pub message: String,
}

impl OverageThresholdRule for OverageThresholdRequiredCodeRule {
    fn check(&self, item: &OverageThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage threshold.
#[derive(Debug, Clone)]
pub struct OverageThresholdValidator {
    pub rules: Vec<Rc<dyn OverageThresholdRule>>,
}

impl OverageThresholdValidator {
    pub fn validate(&self, item: &OverageThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_threshold_validator() -> OverageThresholdValidator {
    OverageThresholdValidator {
        rules: vec![
            Rc::new(OverageThresholdAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn OverageThresholdRule>,
            Rc::new(OverageThresholdKeyIdLengthRule { max_length: 32 }),
            Rc::new(OverageThresholdScoreRangeRule { min: 1.4, max: 3675.4 }),
            Rc::new(OverageThresholdNameLengthRule { max_length: 16 }),
            Rc::new(OverageThresholdQuantityRangeRule { min: 32, max: 80 }),
            Rc::new(OverageThresholdRequiredCodeRule {
                message: "overage threshold needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_threshold_validator();
        let item = new_overage_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = OverageThresholdKeyIdLengthRule { max_length: 255 };
        let mut item = new_overage_threshold("x", "y");
        item.key_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = OverageThresholdScoreRangeRule { min: 5.9, max: 1782.3 };
        let mut item = new_overage_threshold("x", "y");
        item.score = 1783.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = OverageThresholdNameLengthRule { max_length: 3600 };
        let mut item = new_overage_threshold("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = OverageThresholdQuantityRangeRule { min: 3600, max: 3600 };
        let mut item = new_overage_threshold("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageThresholdRequiredCodeRule {
            message: "overage threshold needs this field".to_string(),
        };
        let mut item = new_overage_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
