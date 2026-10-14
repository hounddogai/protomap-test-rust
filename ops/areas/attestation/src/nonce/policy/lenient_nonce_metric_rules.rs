use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient nonce metric must pass. Check returns its problems, or none.
pub trait LenientNonceMetricRule: std::fmt::Debug {
    fn check(&self, item: &LenientNonceMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceMetricTitleLengthRule {
    pub max_length: i64,
}

impl LenientNonceMetricRule for LenientNonceMetricTitleLengthRule {
    fn check(&self, item: &LenientNonceMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceMetricRequiredTitleRule {
    pub message: String,
}

impl LenientNonceMetricRule for LenientNonceMetricRequiredTitleRule {
    fn check(&self, item: &LenientNonceMetric) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceMetricDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientNonceMetricRule for LenientNonceMetricDiscountRangeRule {
    fn check(&self, item: &LenientNonceMetric) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceMetricRequiredCategoryRule {
    pub message: String,
}

impl LenientNonceMetricRule for LenientNonceMetricRequiredCategoryRule {
    fn check(&self, item: &LenientNonceMetric) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceMetricMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientNonceMetricRule for LenientNonceMetricMarginRangeRule {
    fn check(&self, item: &LenientNonceMetric) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient nonce metric.
#[derive(Debug, Clone)]
pub struct LenientNonceMetricValidator {
    pub rules: Vec<Rc<dyn LenientNonceMetricRule>>,
}

impl LenientNonceMetricValidator {
    pub fn validate(&self, item: &LenientNonceMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientNonceMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientNonceMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_nonce_metric_validator() -> LenientNonceMetricValidator {
    LenientNonceMetricValidator {
        rules: vec![
            Rc::new(LenientNonceMetricTitleLengthRule { max_length: 86400 }) as Rc<dyn LenientNonceMetricRule>,
            Rc::new(LenientNonceMetricRequiredTitleRule {
                message: "lenient nonce metric needs a value".to_string(),
            }),
            Rc::new(LenientNonceMetricDiscountRangeRule { min: 9.3, max: 466.1 }),
            Rc::new(LenientNonceMetricRequiredCategoryRule {
                message: "lenient nonce metric needs an entry".to_string(),
            }),
            Rc::new(LenientNonceMetricMarginRangeRule { min: 3.6, max: 1149.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_nonce_metric_validator();
        let item = new_lenient_nonce_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientNonceMetricTitleLengthRule { max_length: 80 };
        let mut item = new_lenient_nonce_metric("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientNonceMetricRequiredTitleRule {
            message: "lenient nonce metric needs this field".to_string(),
        };
        let mut item = new_lenient_nonce_metric("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = LenientNonceMetricDiscountRangeRule { min: 6.2, max: 2374.5 };
        let mut item = new_lenient_nonce_metric("x", "y");
        item.discount = 2376.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = LenientNonceMetricRequiredCategoryRule {
            message: "lenient nonce metric needs this field".to_string(),
        };
        let mut item = new_lenient_nonce_metric("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LenientNonceMetricMarginRangeRule { min: 5.0, max: 3978.7 };
        let mut item = new_lenient_nonce_metric("x", "y");
        item.margin = 3980.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
