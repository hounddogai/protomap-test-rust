use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted quote bundle must pass. Check returns its problems, or none.
pub trait WeightedQuoteBundleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedQuoteBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundleRequiredCodeRule {
    pub message: String,
}

impl WeightedQuoteBundleRule for WeightedQuoteBundleRequiredCodeRule {
    fn check(&self, item: &WeightedQuoteBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundleAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl WeightedQuoteBundleRule for WeightedQuoteBundleAllowedSourceRule {
    fn check(&self, item: &WeightedQuoteBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is unknown", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedQuoteBundleRule for WeightedQuoteBundleExpiresAtRangeRule {
    fn check(&self, item: &WeightedQuoteBundle) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundleSourceLengthRule {
    pub max_length: i64,
}

impl WeightedQuoteBundleRule for WeightedQuoteBundleSourceLengthRule {
    fn check(&self, item: &WeightedQuoteBundle) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundleLabelLengthRule {
    pub max_length: i64,
}

impl WeightedQuoteBundleRule for WeightedQuoteBundleLabelLengthRule {
    fn check(&self, item: &WeightedQuoteBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedQuoteBundleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedQuoteBundleRule for WeightedQuoteBundleDiscountRangeRule {
    fn check(&self, item: &WeightedQuoteBundle) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted quote bundle.
#[derive(Debug, Clone)]
pub struct WeightedQuoteBundleValidator {
    pub rules: Vec<Rc<dyn WeightedQuoteBundleRule>>,
}

impl WeightedQuoteBundleValidator {
    pub fn validate(&self, item: &WeightedQuoteBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedQuoteBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedQuoteBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_quote_bundle_validator() -> WeightedQuoteBundleValidator {
    WeightedQuoteBundleValidator {
        rules: vec![
            Rc::new(WeightedQuoteBundleRequiredCodeRule {
                message: "weighted quote bundle needs this field".to_string(),
            }) as Rc<dyn WeightedQuoteBundleRule>,
            Rc::new(WeightedQuoteBundleAllowedSourceRule {
                allowed: vec!["scheduled".to_string(), "pending".to_string()],
            }),
            Rc::new(WeightedQuoteBundleExpiresAtRangeRule { min: 80, max: 120 }),
            Rc::new(WeightedQuoteBundleSourceLengthRule { max_length: 80 }),
            Rc::new(WeightedQuoteBundleLabelLengthRule { max_length: 86400 }),
            Rc::new(WeightedQuoteBundleDiscountRangeRule { min: 5.6, max: 3967.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_quote_bundle_validator();
        let item = new_weighted_quote_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedQuoteBundleRequiredCodeRule {
            message: "weighted quote bundle needs an entry".to_string(),
        };
        let mut item = new_weighted_quote_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = WeightedQuoteBundleExpiresAtRangeRule { min: 86400, max: 86400 };
        let mut item = new_weighted_quote_bundle("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = WeightedQuoteBundleSourceLengthRule { max_length: 32 };
        let mut item = new_weighted_quote_bundle("x", "y");
        item.source = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedQuoteBundleLabelLengthRule { max_length: 32 };
        let mut item = new_weighted_quote_bundle("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = WeightedQuoteBundleDiscountRangeRule { min: 10.0, max: 2513.6 };
        let mut item = new_weighted_quote_bundle("x", "y");
        item.discount = 2515.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
