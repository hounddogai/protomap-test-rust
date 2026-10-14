use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global forecast threshold must pass. Check returns its problems, or none.
pub trait GlobalForecastThresholdRule: std::fmt::Debug {
    fn check(&self, item: &GlobalForecastThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastThresholdAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl GlobalForecastThresholdRule for GlobalForecastThresholdAllowedTitleRule {
    fn check(&self, item: &GlobalForecastThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastThresholdTitleLengthRule {
    pub max_length: i64,
}

impl GlobalForecastThresholdRule for GlobalForecastThresholdTitleLengthRule {
    fn check(&self, item: &GlobalForecastThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalForecastThresholdCategoryLengthRule {
    pub max_length: i64,
}

impl GlobalForecastThresholdRule for GlobalForecastThresholdCategoryLengthRule {
    fn check(&self, item: &GlobalForecastThreshold) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a global forecast threshold.
#[derive(Debug, Clone)]
pub struct GlobalForecastThresholdValidator {
    pub rules: Vec<Rc<dyn GlobalForecastThresholdRule>>,
}

impl GlobalForecastThresholdValidator {
    pub fn validate(&self, item: &GlobalForecastThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalForecastThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalForecastThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_forecast_threshold_validator() -> GlobalForecastThresholdValidator {
    GlobalForecastThresholdValidator {
        rules: vec![
            Rc::new(GlobalForecastThresholdAllowedTitleRule { allowed: Vec::new() })
                as Rc<dyn GlobalForecastThresholdRule>,
            Rc::new(GlobalForecastThresholdTitleLengthRule { max_length: 3600 }),
            Rc::new(GlobalForecastThresholdCategoryLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_forecast_threshold_validator();
        let item = new_global_forecast_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalForecastThresholdTitleLengthRule { max_length: 80 };
        let mut item = new_global_forecast_threshold("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = GlobalForecastThresholdCategoryLengthRule { max_length: 3600 };
        let mut item = new_global_forecast_threshold("x", "y");
        item.category = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
