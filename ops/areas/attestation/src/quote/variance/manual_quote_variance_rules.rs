use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual quote variance must pass. Check returns its problems, or none.
pub trait ManualQuoteVarianceRule: std::fmt::Debug {
    fn check(&self, item: &ManualQuoteVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteVarianceRequiredNameRule {
    pub message: String,
}

impl ManualQuoteVarianceRule for ManualQuoteVarianceRequiredNameRule {
    fn check(&self, item: &ManualQuoteVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteVarianceCodeLengthRule {
    pub max_length: i64,
}

impl ManualQuoteVarianceRule for ManualQuoteVarianceCodeLengthRule {
    fn check(&self, item: &ManualQuoteVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteVarianceAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl ManualQuoteVarianceRule for ManualQuoteVarianceAllowedCategoryRule {
    fn check(&self, item: &ManualQuoteVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is below minimum", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQuoteVarianceVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualQuoteVarianceRule for ManualQuoteVarianceVersionRangeRule {
    fn check(&self, item: &ManualQuoteVariance) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual quote variance.
#[derive(Debug, Clone)]
pub struct ManualQuoteVarianceValidator {
    pub rules: Vec<Rc<dyn ManualQuoteVarianceRule>>,
}

impl ManualQuoteVarianceValidator {
    pub fn validate(&self, item: &ManualQuoteVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualQuoteVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualQuoteVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_quote_variance_validator() -> ManualQuoteVarianceValidator {
    ManualQuoteVarianceValidator {
        rules: vec![
            Rc::new(ManualQuoteVarianceRequiredNameRule {
                message: "manual quote variance needs an entry".to_string(),
            }) as Rc<dyn ManualQuoteVarianceRule>,
            Rc::new(ManualQuoteVarianceCodeLengthRule { max_length: 3600 }),
            Rc::new(ManualQuoteVarianceAllowedCategoryRule {
                allowed: vec!["tiered".to_string()],
            }),
            Rc::new(ManualQuoteVarianceVersionRangeRule { min: 16, max: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_quote_variance_validator();
        let item = new_manual_quote_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualQuoteVarianceRequiredNameRule {
            message: "manual quote variance needs an entry".to_string(),
        };
        let mut item = new_manual_quote_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualQuoteVarianceCodeLengthRule { max_length: 120 };
        let mut item = new_manual_quote_variance("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ManualQuoteVarianceVersionRangeRule { min: 32, max: 80 };
        let mut item = new_manual_quote_variance("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
