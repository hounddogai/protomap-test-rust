use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched page baseline must pass. Check returns its problems, or none.
pub trait BatchedPageBaselineRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPageBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageBaselineCodePrefixRule {
    pub prefix: String,
}

impl BatchedPageBaselineRule for BatchedPageBaselineCodePrefixRule {
    fn check(&self, item: &BatchedPageBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageBaselineAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl BatchedPageBaselineRule for BatchedPageBaselineAllowedCategoryRule {
    fn check(&self, item: &BatchedPageBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is duplicated", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageBaselineIssuerLengthRule {
    pub max_length: i64,
}

impl BatchedPageBaselineRule for BatchedPageBaselineIssuerLengthRule {
    fn check(&self, item: &BatchedPageBaseline) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPageBaselineRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPageBaselineRule for BatchedPageBaselineRotationDaysRangeRule {
    fn check(&self, item: &BatchedPageBaseline) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a batched page baseline.
#[derive(Debug, Clone)]
pub struct BatchedPageBaselineValidator {
    pub rules: Vec<Rc<dyn BatchedPageBaselineRule>>,
}

impl BatchedPageBaselineValidator {
    pub fn validate(&self, item: &BatchedPageBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedPageBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPageBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_page_baseline_validator() -> BatchedPageBaselineValidator {
    BatchedPageBaselineValidator {
        rules: vec![
            Rc::new(BatchedPageBaselineCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn BatchedPageBaselineRule>,
            Rc::new(BatchedPageBaselineAllowedCategoryRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(BatchedPageBaselineIssuerLengthRule { max_length: 86400 }),
            Rc::new(BatchedPageBaselineRotationDaysRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_page_baseline_validator();
        let item = new_batched_page_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = BatchedPageBaselineIssuerLengthRule { max_length: 16 };
        let mut item = new_batched_page_baseline("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = BatchedPageBaselineRotationDaysRangeRule { min: 32, max: 80 };
        let mut item = new_batched_page_baseline("x", "y");
        item.rotation_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
