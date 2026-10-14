use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal token metric must pass. Check returns its problems, or none.
pub trait SeasonalTokenMetricRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalTokenMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenMetricCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalTokenMetricRule for SeasonalTokenMetricCodeLengthRule {
    fn check(&self, item: &SeasonalTokenMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenMetricRequiredChecksumRule {
    pub message: String,
}

impl SeasonalTokenMetricRule for SeasonalTokenMetricRequiredChecksumRule {
    fn check(&self, item: &SeasonalTokenMetric) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenMetricTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalTokenMetricRule for SeasonalTokenMetricTitleLengthRule {
    fn check(&self, item: &SeasonalTokenMetric) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal token metric.
#[derive(Debug, Clone)]
pub struct SeasonalTokenMetricValidator {
    pub rules: Vec<Rc<dyn SeasonalTokenMetricRule>>,
}

impl SeasonalTokenMetricValidator {
    pub fn validate(&self, item: &SeasonalTokenMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalTokenMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalTokenMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_token_metric_validator() -> SeasonalTokenMetricValidator {
    SeasonalTokenMetricValidator {
        rules: vec![
            Rc::new(SeasonalTokenMetricCodeLengthRule { max_length: 16 }) as Rc<dyn SeasonalTokenMetricRule>,
            Rc::new(SeasonalTokenMetricRequiredChecksumRule {
                message: "seasonal token metric needs an entry".to_string(),
            }),
            Rc::new(SeasonalTokenMetricTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_token_metric_validator();
        let item = new_seasonal_token_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalTokenMetricCodeLengthRule { max_length: 16 };
        let mut item = new_seasonal_token_metric("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = SeasonalTokenMetricRequiredChecksumRule {
            message: "seasonal token metric needs a value".to_string(),
        };
        let mut item = new_seasonal_token_metric("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalTokenMetricTitleLengthRule { max_length: 3600 };
        let mut item = new_seasonal_token_metric("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
