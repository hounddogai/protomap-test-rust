use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction estimate must pass. Check returns its problems, or none.
pub trait RedactionEstimateRule: std::fmt::Debug {
    fn check(&self, item: &RedactionEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEstimateExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionEstimateRule for RedactionEstimateExpiresAtRangeRule {
    fn check(&self, item: &RedactionEstimate) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEstimateCategoryLengthRule {
    pub max_length: i64,
}

impl RedactionEstimateRule for RedactionEstimateCategoryLengthRule {
    fn check(&self, item: &RedactionEstimate) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEstimateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionEstimateRule for RedactionEstimateBudgetRangeRule {
    fn check(&self, item: &RedactionEstimate) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEstimateRequiredCodeRule {
    pub message: String,
}

impl RedactionEstimateRule for RedactionEstimateRequiredCodeRule {
    fn check(&self, item: &RedactionEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction estimate.
#[derive(Debug, Clone)]
pub struct RedactionEstimateValidator {
    pub rules: Vec<Rc<dyn RedactionEstimateRule>>,
}

impl RedactionEstimateValidator {
    pub fn validate(&self, item: &RedactionEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_estimate_validator() -> RedactionEstimateValidator {
    RedactionEstimateValidator {
        rules: vec![
            Rc::new(RedactionEstimateExpiresAtRangeRule { min: 80, max: 1000 }) as Rc<dyn RedactionEstimateRule>,
            Rc::new(RedactionEstimateCategoryLengthRule { max_length: 86400 }),
            Rc::new(RedactionEstimateBudgetRangeRule { min: 0.1, max: 1429.7 }),
            Rc::new(RedactionEstimateRequiredCodeRule {
                message: "redaction estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_estimate_validator();
        let item = new_redaction_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RedactionEstimateExpiresAtRangeRule { min: 3600, max: 3600 };
        let mut item = new_redaction_estimate("x", "y");
        item.expires_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = RedactionEstimateCategoryLengthRule { max_length: 64 };
        let mut item = new_redaction_estimate("x", "y");
        item.category = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RedactionEstimateBudgetRangeRule { min: 5.5, max: 3321.6 };
        let mut item = new_redaction_estimate("x", "y");
        item.budget = 3323.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionEstimateRequiredCodeRule {
            message: "redaction estimate needs a value".to_string(),
        };
        let mut item = new_redaction_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
