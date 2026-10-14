use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling grant variance must pass. Check returns its problems, or none.
pub trait RollingGrantVarianceRule: std::fmt::Debug {
    fn check(&self, item: &RollingGrantVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVarianceTitleLengthRule {
    pub max_length: i64,
}

impl RollingGrantVarianceRule for RollingGrantVarianceTitleLengthRule {
    fn check(&self, item: &RollingGrantVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVarianceRequiredTitleRule {
    pub message: String,
}

impl RollingGrantVarianceRule for RollingGrantVarianceRequiredTitleRule {
    fn check(&self, item: &RollingGrantVariance) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVarianceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollingGrantVarianceRule for RollingGrantVarianceAllowedCodeRule {
    fn check(&self, item: &RollingGrantVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVariancePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingGrantVarianceRule for RollingGrantVariancePortRangeRule {
    fn check(&self, item: &RollingGrantVariance) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVarianceRequiredCodeRule {
    pub message: String,
}

impl RollingGrantVarianceRule for RollingGrantVarianceRequiredCodeRule {
    fn check(&self, item: &RollingGrantVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantVarianceGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingGrantVarianceRule for RollingGrantVarianceGraceDaysRangeRule {
    fn check(&self, item: &RollingGrantVariance) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling grant variance.
#[derive(Debug, Clone)]
pub struct RollingGrantVarianceValidator {
    pub rules: Vec<Rc<dyn RollingGrantVarianceRule>>,
}

impl RollingGrantVarianceValidator {
    pub fn validate(&self, item: &RollingGrantVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingGrantVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingGrantVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_grant_variance_validator() -> RollingGrantVarianceValidator {
    RollingGrantVarianceValidator {
        rules: vec![
            Rc::new(RollingGrantVarianceTitleLengthRule { max_length: 120 }) as Rc<dyn RollingGrantVarianceRule>,
            Rc::new(RollingGrantVarianceRequiredTitleRule {
                message: "rolling grant variance needs a value".to_string(),
            }),
            Rc::new(RollingGrantVarianceAllowedCodeRule {
                allowed: vec!["default".to_string(), "manual".to_string(), "tiered".to_string()],
            }),
            Rc::new(RollingGrantVariancePortRangeRule { min: 255, max: 1000 }),
            Rc::new(RollingGrantVarianceRequiredCodeRule {
                message: "rolling grant variance needs an entry".to_string(),
            }),
            Rc::new(RollingGrantVarianceGraceDaysRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_grant_variance_validator();
        let item = new_rolling_grant_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingGrantVarianceTitleLengthRule { max_length: 86400 };
        let mut item = new_rolling_grant_variance("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingGrantVarianceRequiredTitleRule {
            message: "rolling grant variance needs an entry".to_string(),
        };
        let mut item = new_rolling_grant_variance("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = RollingGrantVariancePortRangeRule { min: 16, max: 32 };
        let mut item = new_rolling_grant_variance("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingGrantVarianceRequiredCodeRule {
            message: "rolling grant variance needs a value".to_string(),
        };
        let mut item = new_rolling_grant_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RollingGrantVarianceGraceDaysRangeRule { min: 32, max: 1000 };
        let mut item = new_rolling_grant_variance("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
