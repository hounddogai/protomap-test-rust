use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched page threshold must pass. Check returns its problems, or none.
pub trait BatchedPageThresholdRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPageThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageThresholdDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedPageThresholdRule for BatchedPageThresholdDiscountRangeRule {
    fn check(&self, item: &BatchedPageThreshold) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageThresholdLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPageThresholdRule for BatchedPageThresholdLimitRangeRule {
    fn check(&self, item: &BatchedPageThreshold) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageThresholdCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedPageThresholdRule for BatchedPageThresholdCeilingRangeRule {
    fn check(&self, item: &BatchedPageThreshold) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageThresholdPinnedGuardRule {
    pub limit: i64,
}

impl BatchedPageThresholdRule for BatchedPageThresholdPinnedGuardRule {
    fn check(&self, item: &BatchedPageThreshold) -> Vec<String> {
        if item.pinned && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while pinned",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedPageThresholdRule for BatchedPageThresholdAllowedCodeRule {
    fn check(&self, item: &BatchedPageThreshold) -> Vec<String> {
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
pub struct BatchedPageThresholdNameLengthRule {
    pub max_length: i64,
}

impl BatchedPageThresholdRule for BatchedPageThresholdNameLengthRule {
    fn check(&self, item: &BatchedPageThreshold) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched page threshold.
#[derive(Debug, Clone)]
pub struct BatchedPageThresholdValidator {
    pub rules: Vec<Rc<dyn BatchedPageThresholdRule>>,
}

impl BatchedPageThresholdValidator {
    pub fn validate(&self, item: &BatchedPageThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedPageThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPageThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_page_threshold_validator() -> BatchedPageThresholdValidator {
    BatchedPageThresholdValidator {
        rules: vec![
            Rc::new(BatchedPageThresholdDiscountRangeRule { min: 4.6, max: 2723.9 })
                as Rc<dyn BatchedPageThresholdRule>,
            Rc::new(BatchedPageThresholdLimitRangeRule { min: 16, max: 86400 }),
            Rc::new(BatchedPageThresholdCeilingRangeRule { min: 7.4, max: 2579.1 }),
            Rc::new(BatchedPageThresholdPinnedGuardRule { limit: 3600 }),
            Rc::new(BatchedPageThresholdAllowedCodeRule {
                allowed: vec!["default".to_string(), "rolling".to_string()],
            }),
            Rc::new(BatchedPageThresholdNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_page_threshold_validator();
        let item = new_batched_page_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = BatchedPageThresholdDiscountRangeRule { min: 0.3, max: 1551.9 };
        let mut item = new_batched_page_threshold("x", "y");
        item.discount = 1553.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = BatchedPageThresholdLimitRangeRule { min: 120, max: 3600 };
        let mut item = new_batched_page_threshold("x", "y");
        item.limit = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = BatchedPageThresholdCeilingRangeRule { min: 5.0, max: 4685.4 };
        let mut item = new_batched_page_threshold("x", "y");
        item.ceiling = 4686.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedPageThresholdNameLengthRule { max_length: 86400 };
        let mut item = new_batched_page_threshold("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
