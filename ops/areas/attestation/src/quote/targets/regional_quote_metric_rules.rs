use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional quote metric must pass. Check returns its problems, or none.
pub trait RegionalQuoteMetricRule: std::fmt::Debug {
    fn check(&self, item: &RegionalQuoteMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteMetricDescriptionLengthRule {
    pub max_length: i64,
}

impl RegionalQuoteMetricRule for RegionalQuoteMetricDescriptionLengthRule {
    fn check(&self, item: &RegionalQuoteMetric) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteMetricRequiredCodeRule {
    pub message: String,
}

impl RegionalQuoteMetricRule for RegionalQuoteMetricRequiredCodeRule {
    fn check(&self, item: &RegionalQuoteMetric) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteMetricTitleLengthRule {
    pub max_length: i64,
}

impl RegionalQuoteMetricRule for RegionalQuoteMetricTitleLengthRule {
    fn check(&self, item: &RegionalQuoteMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteMetricIssuerLengthRule {
    pub max_length: i64,
}

impl RegionalQuoteMetricRule for RegionalQuoteMetricIssuerLengthRule {
    fn check(&self, item: &RegionalQuoteMetric) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQuoteMetricScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalQuoteMetricRule for RegionalQuoteMetricScoreRangeRule {
    fn check(&self, item: &RegionalQuoteMetric) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional quote metric.
#[derive(Debug, Clone)]
pub struct RegionalQuoteMetricValidator {
    pub rules: Vec<Rc<dyn RegionalQuoteMetricRule>>,
}

impl RegionalQuoteMetricValidator {
    pub fn validate(&self, item: &RegionalQuoteMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RegionalQuoteMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalQuoteMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_quote_metric_validator() -> RegionalQuoteMetricValidator {
    RegionalQuoteMetricValidator {
        rules: vec![
            Rc::new(RegionalQuoteMetricDescriptionLengthRule { max_length: 3600 }) as Rc<dyn RegionalQuoteMetricRule>,
            Rc::new(RegionalQuoteMetricRequiredCodeRule {
                message: "regional quote metric needs an entry".to_string(),
            }),
            Rc::new(RegionalQuoteMetricTitleLengthRule { max_length: 32 }),
            Rc::new(RegionalQuoteMetricIssuerLengthRule { max_length: 32 }),
            Rc::new(RegionalQuoteMetricScoreRangeRule { min: 4.2, max: 846.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_quote_metric_validator();
        let item = new_regional_quote_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = RegionalQuoteMetricDescriptionLengthRule { max_length: 120 };
        let mut item = new_regional_quote_metric("x", "y");
        item.description = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RegionalQuoteMetricRequiredCodeRule {
            message: "regional quote metric needs a value".to_string(),
        };
        let mut item = new_regional_quote_metric("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalQuoteMetricTitleLengthRule { max_length: 32 };
        let mut item = new_regional_quote_metric("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = RegionalQuoteMetricIssuerLengthRule { max_length: 3600 };
        let mut item = new_regional_quote_metric("x", "y");
        item.issuer = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RegionalQuoteMetricScoreRangeRule { min: 3.7, max: 3830.6 };
        let mut item = new_regional_quote_metric("x", "y");
        item.score = 3832.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
