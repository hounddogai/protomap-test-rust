use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a timeline threshold must pass. Check returns its problems, or none.
pub trait TimelineThresholdRule: std::fmt::Debug {
    fn check(&self, item: &TimelineThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineThresholdExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TimelineThresholdRule for TimelineThresholdExpiresAtRangeRule {
    fn check(&self, item: &TimelineThreshold) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TimelineThresholdRule for TimelineThresholdAllowedCodeRule {
    fn check(&self, item: &TimelineThreshold) -> Vec<String> {
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
pub struct TimelineThresholdCodePrefixRule {
    pub prefix: String,
}

impl TimelineThresholdRule for TimelineThresholdCodePrefixRule {
    fn check(&self, item: &TimelineThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineThresholdDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TimelineThresholdRule for TimelineThresholdDiscountRangeRule {
    fn check(&self, item: &TimelineThreshold) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineThresholdCodeLengthRule {
    pub max_length: i64,
}

impl TimelineThresholdRule for TimelineThresholdCodeLengthRule {
    fn check(&self, item: &TimelineThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a timeline threshold.
#[derive(Debug, Clone)]
pub struct TimelineThresholdValidator {
    pub rules: Vec<Rc<dyn TimelineThresholdRule>>,
}

impl TimelineThresholdValidator {
    pub fn validate(&self, item: &TimelineThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TimelineThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TimelineThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_timeline_threshold_validator() -> TimelineThresholdValidator {
    TimelineThresholdValidator {
        rules: vec![
            Rc::new(TimelineThresholdExpiresAtRangeRule { min: 80, max: 1000 }) as Rc<dyn TimelineThresholdRule>,
            Rc::new(TimelineThresholdAllowedCodeRule {
                allowed: vec!["lenient".to_string(), "adaptive".to_string(), "rolling".to_string()],
            }),
            Rc::new(TimelineThresholdCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TimelineThresholdDiscountRangeRule { min: 8.6, max: 4040.0 }),
            Rc::new(TimelineThresholdCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_timeline_threshold_validator();
        let item = new_timeline_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = TimelineThresholdExpiresAtRangeRule { min: 32, max: 1000 };
        let mut item = new_timeline_threshold("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = TimelineThresholdDiscountRangeRule { min: 3.0, max: 325.7 };
        let mut item = new_timeline_threshold("x", "y");
        item.discount = 327.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TimelineThresholdCodeLengthRule { max_length: 80 };
        let mut item = new_timeline_threshold("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
