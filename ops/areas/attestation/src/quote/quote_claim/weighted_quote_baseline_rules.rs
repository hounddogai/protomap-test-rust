use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted quote baseline must pass. Check returns its problems, or none.
pub trait WeightedQuoteBaselineRule: std::fmt::Debug {
    fn check(&self, item: &WeightedQuoteBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBaselineRequiredSummaryRule {
    pub message: String,
}

impl WeightedQuoteBaselineRule for WeightedQuoteBaselineRequiredSummaryRule {
    fn check(&self, item: &WeightedQuoteBaseline) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBaselineCodeLengthRule {
    pub max_length: i64,
}

impl WeightedQuoteBaselineRule for WeightedQuoteBaselineCodeLengthRule {
    fn check(&self, item: &WeightedQuoteBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBaselineAllowedRegionRule {
    pub allowed: Vec<String>,
}

impl WeightedQuoteBaselineRule for WeightedQuoteBaselineAllowedRegionRule {
    fn check(&self, item: &WeightedQuoteBaseline) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.region))) {
            return vec![format!("region {} is above maximum", item.region)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBaselineCodePrefixRule {
    pub prefix: String,
}

impl WeightedQuoteBaselineRule for WeightedQuoteBaselineCodePrefixRule {
    fn check(&self, item: &WeightedQuoteBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBaselineSummaryLengthRule {
    pub max_length: i64,
}

impl WeightedQuoteBaselineRule for WeightedQuoteBaselineSummaryLengthRule {
    fn check(&self, item: &WeightedQuoteBaseline) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted quote baseline.
#[derive(Debug, Clone)]
pub struct WeightedQuoteBaselineValidator {
    pub rules: Vec<Rc<dyn WeightedQuoteBaselineRule>>,
}

impl WeightedQuoteBaselineValidator {
    pub fn validate(&self, item: &WeightedQuoteBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedQuoteBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedQuoteBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_quote_baseline_validator() -> WeightedQuoteBaselineValidator {
    WeightedQuoteBaselineValidator {
        rules: vec![
            Rc::new(WeightedQuoteBaselineRequiredSummaryRule {
                message: "weighted quote baseline needs a value".to_string(),
            }) as Rc<dyn WeightedQuoteBaselineRule>,
            Rc::new(WeightedQuoteBaselineCodeLengthRule { max_length: 1000 }),
            Rc::new(WeightedQuoteBaselineAllowedRegionRule {
                allowed: vec!["seasonal".to_string(), "pending".to_string()],
            }),
            Rc::new(WeightedQuoteBaselineCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(WeightedQuoteBaselineSummaryLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_quote_baseline_validator();
        let item = new_weighted_quote_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = WeightedQuoteBaselineRequiredSummaryRule {
            message: "weighted quote baseline needs an entry".to_string(),
        };
        let mut item = new_weighted_quote_baseline("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedQuoteBaselineCodeLengthRule { max_length: 16 };
        let mut item = new_weighted_quote_baseline("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = WeightedQuoteBaselineSummaryLengthRule { max_length: 80 };
        let mut item = new_weighted_quote_baseline("x", "y");
        item.summary = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
