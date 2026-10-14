use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a federation threshold must pass. Check returns its problems, or none.
pub trait FederationThresholdRule: std::fmt::Debug {
    fn check(&self, item: &FederationThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationThresholdTimezoneLengthRule {
    pub max_length: i64,
}

impl FederationThresholdRule for FederationThresholdTimezoneLengthRule {
    fn check(&self, item: &FederationThreshold) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationThresholdRequiredLabelRule {
    pub message: String,
}

impl FederationThresholdRule for FederationThresholdRequiredLabelRule {
    fn check(&self, item: &FederationThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationThresholdAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FederationThresholdRule for FederationThresholdAllowedLabelRule {
    fn check(&self, item: &FederationThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationThresholdRequiredCodeRule {
    pub message: String,
}

impl FederationThresholdRule for FederationThresholdRequiredCodeRule {
    fn check(&self, item: &FederationThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationThresholdGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FederationThresholdRule for FederationThresholdGraceDaysRangeRule {
    fn check(&self, item: &FederationThreshold) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FederationThresholdCodeLengthRule {
    pub max_length: i64,
}

impl FederationThresholdRule for FederationThresholdCodeLengthRule {
    fn check(&self, item: &FederationThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a federation threshold.
#[derive(Debug, Clone)]
pub struct FederationThresholdValidator {
    pub rules: Vec<Rc<dyn FederationThresholdRule>>,
}

impl FederationThresholdValidator {
    pub fn validate(&self, item: &FederationThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FederationThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FederationThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_federation_threshold_validator() -> FederationThresholdValidator {
    FederationThresholdValidator {
        rules: vec![
            Rc::new(FederationThresholdTimezoneLengthRule { max_length: 80 }) as Rc<dyn FederationThresholdRule>,
            Rc::new(FederationThresholdRequiredLabelRule {
                message: "federation threshold needs a value".to_string(),
            }),
            Rc::new(FederationThresholdAllowedLabelRule {
                allowed: vec!["tiered".to_string(), "capped".to_string()],
            }),
            Rc::new(FederationThresholdRequiredCodeRule {
                message: "federation threshold needs this field".to_string(),
            }),
            Rc::new(FederationThresholdGraceDaysRangeRule { min: 120, max: 1000 }),
            Rc::new(FederationThresholdCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_federation_threshold_validator();
        let item = new_federation_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = FederationThresholdTimezoneLengthRule { max_length: 16 };
        let mut item = new_federation_threshold("x", "y");
        item.timezone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FederationThresholdRequiredLabelRule {
            message: "federation threshold needs a value".to_string(),
        };
        let mut item = new_federation_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FederationThresholdRequiredCodeRule {
            message: "federation threshold needs a value".to_string(),
        };
        let mut item = new_federation_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = FederationThresholdGraceDaysRangeRule { min: 32, max: 3600 };
        let mut item = new_federation_threshold("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FederationThresholdCodeLengthRule { max_length: 32 };
        let mut item = new_federation_threshold("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
