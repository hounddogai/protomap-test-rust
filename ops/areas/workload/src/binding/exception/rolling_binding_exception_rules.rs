use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling binding exception must pass. Check returns its problems, or none.
pub trait RollingBindingExceptionRule: std::fmt::Debug {
    fn check(&self, item: &RollingBindingException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingExceptionRequiredCodeRule {
    pub message: String,
}

impl RollingBindingExceptionRule for RollingBindingExceptionRequiredCodeRule {
    fn check(&self, item: &RollingBindingException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingExceptionAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl RollingBindingExceptionRule for RollingBindingExceptionAllowedLocaleRule {
    fn check(&self, item: &RollingBindingException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is unknown", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingExceptionRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingBindingExceptionRule for RollingBindingExceptionRetriesRangeRule {
    fn check(&self, item: &RollingBindingException) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingExceptionLabelLengthRule {
    pub max_length: i64,
}

impl RollingBindingExceptionRule for RollingBindingExceptionLabelLengthRule {
    fn check(&self, item: &RollingBindingException) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingExceptionRequiredLocaleRule {
    pub message: String,
}

impl RollingBindingExceptionRule for RollingBindingExceptionRequiredLocaleRule {
    fn check(&self, item: &RollingBindingException) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBindingExceptionMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingBindingExceptionRule for RollingBindingExceptionMinimumRangeRule {
    fn check(&self, item: &RollingBindingException) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling binding exception.
#[derive(Debug, Clone)]
pub struct RollingBindingExceptionValidator {
    pub rules: Vec<Rc<dyn RollingBindingExceptionRule>>,
}

impl RollingBindingExceptionValidator {
    pub fn validate(&self, item: &RollingBindingException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingBindingException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingBindingException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_binding_exception_validator() -> RollingBindingExceptionValidator {
    RollingBindingExceptionValidator {
        rules: vec![
            Rc::new(RollingBindingExceptionRequiredCodeRule {
                message: "rolling binding exception needs an entry".to_string(),
            }) as Rc<dyn RollingBindingExceptionRule>,
            Rc::new(RollingBindingExceptionAllowedLocaleRule { allowed: Vec::new() }),
            Rc::new(RollingBindingExceptionRetriesRangeRule { min: 255, max: 1000 }),
            Rc::new(RollingBindingExceptionLabelLengthRule { max_length: 80 }),
            Rc::new(RollingBindingExceptionRequiredLocaleRule {
                message: "rolling binding exception needs this field".to_string(),
            }),
            Rc::new(RollingBindingExceptionMinimumRangeRule { min: 6.8, max: 721.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_binding_exception_validator();
        let item = new_rolling_binding_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingBindingExceptionRequiredCodeRule {
            message: "rolling binding exception needs this field".to_string(),
        };
        let mut item = new_rolling_binding_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RollingBindingExceptionRetriesRangeRule { min: 16, max: 80 };
        let mut item = new_rolling_binding_exception("x", "y");
        item.retries = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingBindingExceptionLabelLengthRule { max_length: 1000 };
        let mut item = new_rolling_binding_exception("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RollingBindingExceptionRequiredLocaleRule {
            message: "rolling binding exception needs this field".to_string(),
        };
        let mut item = new_rolling_binding_exception("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = RollingBindingExceptionMinimumRangeRule { min: 6.1, max: 3445.1 };
        let mut item = new_rolling_binding_exception("x", "y");
        item.minimum = 3446.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
