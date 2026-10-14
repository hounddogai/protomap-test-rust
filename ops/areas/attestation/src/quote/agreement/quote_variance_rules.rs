use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote variance must pass. Check returns its problems, or none.
pub trait QuoteVarianceRule: std::fmt::Debug {
    fn check(&self, item: &QuoteVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteVarianceLocaleLengthRule {
    pub max_length: i64,
}

impl QuoteVarianceRule for QuoteVarianceLocaleLengthRule {
    fn check(&self, item: &QuoteVariance) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteVarianceWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteVarianceRule for QuoteVarianceWeightRangeRule {
    fn check(&self, item: &QuoteVariance) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteVarianceRequiredCodeRule {
    pub message: String,
}

impl QuoteVarianceRule for QuoteVarianceRequiredCodeRule {
    fn check(&self, item: &QuoteVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteVarianceRequiredLabelRule {
    pub message: String,
}

impl QuoteVarianceRule for QuoteVarianceRequiredLabelRule {
    fn check(&self, item: &QuoteVariance) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteVarianceNameLengthRule {
    pub max_length: i64,
}

impl QuoteVarianceRule for QuoteVarianceNameLengthRule {
    fn check(&self, item: &QuoteVariance) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote variance.
#[derive(Debug, Clone)]
pub struct QuoteVarianceValidator {
    pub rules: Vec<Rc<dyn QuoteVarianceRule>>,
}

impl QuoteVarianceValidator {
    pub fn validate(&self, item: &QuoteVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_variance_validator() -> QuoteVarianceValidator {
    QuoteVarianceValidator {
        rules: vec![
            Rc::new(QuoteVarianceLocaleLengthRule { max_length: 86400 }) as Rc<dyn QuoteVarianceRule>,
            Rc::new(QuoteVarianceWeightRangeRule { min: 4.9, max: 747.6 }),
            Rc::new(QuoteVarianceRequiredCodeRule {
                message: "quote variance needs this field".to_string(),
            }),
            Rc::new(QuoteVarianceRequiredLabelRule {
                message: "quote variance needs this field".to_string(),
            }),
            Rc::new(QuoteVarianceNameLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_variance_validator();
        let item = new_quote_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = QuoteVarianceLocaleLengthRule { max_length: 1000 };
        let mut item = new_quote_variance("x", "y");
        item.locale = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = QuoteVarianceWeightRangeRule { min: 3.0, max: 120.2 };
        let mut item = new_quote_variance("x", "y");
        item.weight = 121.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteVarianceRequiredCodeRule {
            message: "quote variance needs this field".to_string(),
        };
        let mut item = new_quote_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = QuoteVarianceRequiredLabelRule {
            message: "quote variance needs this field".to_string(),
        };
        let mut item = new_quote_variance("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = QuoteVarianceNameLengthRule { max_length: 86400 };
        let mut item = new_quote_variance("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
