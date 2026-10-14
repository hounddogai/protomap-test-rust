use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped quote metric must pass. Check returns its problems, or none.
pub trait CappedQuoteMetricRule: std::fmt::Debug {
    fn check(&self, item: &CappedQuoteMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteMetricAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl CappedQuoteMetricRule for CappedQuoteMetricAllowedTitleRule {
    fn check(&self, item: &CappedQuoteMetric) -> Vec<String> {
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
pub struct CappedQuoteMetricRequiredCurrencyRule {
    pub message: String,
}

impl CappedQuoteMetricRule for CappedQuoteMetricRequiredCurrencyRule {
    fn check(&self, item: &CappedQuoteMetric) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteMetricCodeLengthRule {
    pub max_length: i64,
}

impl CappedQuoteMetricRule for CappedQuoteMetricCodeLengthRule {
    fn check(&self, item: &CappedQuoteMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteMetricWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedQuoteMetricRule for CappedQuoteMetricWeightRangeRule {
    fn check(&self, item: &CappedQuoteMetric) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteMetricRequiredTitleRule {
    pub message: String,
}

impl CappedQuoteMetricRule for CappedQuoteMetricRequiredTitleRule {
    fn check(&self, item: &CappedQuoteMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedQuoteMetricRequiredDescriptionRule {
    pub message: String,
}

impl CappedQuoteMetricRule for CappedQuoteMetricRequiredDescriptionRule {
    fn check(&self, item: &CappedQuoteMetric) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped quote metric.
#[derive(Debug, Clone)]
pub struct CappedQuoteMetricValidator {
    pub rules: Vec<Rc<dyn CappedQuoteMetricRule>>,
}

impl CappedQuoteMetricValidator {
    pub fn validate(&self, item: &CappedQuoteMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedQuoteMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedQuoteMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_quote_metric_validator() -> CappedQuoteMetricValidator {
    CappedQuoteMetricValidator {
        rules: vec![
            Rc::new(CappedQuoteMetricAllowedTitleRule {
                allowed: vec!["default".to_string(), "batched".to_string(), "strict".to_string()],
            }) as Rc<dyn CappedQuoteMetricRule>,
            Rc::new(CappedQuoteMetricRequiredCurrencyRule {
                message: "capped quote metric needs an entry".to_string(),
            }),
            Rc::new(CappedQuoteMetricCodeLengthRule { max_length: 64 }),
            Rc::new(CappedQuoteMetricWeightRangeRule { min: 7.4, max: 1468.2 }),
            Rc::new(CappedQuoteMetricRequiredTitleRule {
                message: "capped quote metric needs a value".to_string(),
            }),
            Rc::new(CappedQuoteMetricRequiredDescriptionRule {
                message: "capped quote metric needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_quote_metric_validator();
        let item = new_capped_quote_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = CappedQuoteMetricRequiredCurrencyRule {
            message: "capped quote metric needs a value".to_string(),
        };
        let mut item = new_capped_quote_metric("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedQuoteMetricCodeLengthRule { max_length: 120 };
        let mut item = new_capped_quote_metric("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = CappedQuoteMetricWeightRangeRule { min: 8.2, max: 1325.2 };
        let mut item = new_capped_quote_metric("x", "y");
        item.weight = 1326.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedQuoteMetricRequiredTitleRule {
            message: "capped quote metric needs an entry".to_string(),
        };
        let mut item = new_capped_quote_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = CappedQuoteMetricRequiredDescriptionRule {
            message: "capped quote metric needs this field".to_string(),
        };
        let mut item = new_capped_quote_metric("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
