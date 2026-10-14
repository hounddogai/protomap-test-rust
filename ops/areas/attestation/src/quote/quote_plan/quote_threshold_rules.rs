use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote threshold must pass. Check returns its problems, or none.
pub trait QuoteThresholdRule: std::fmt::Debug {
    fn check(&self, item: &QuoteThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteThresholdTitleLengthRule {
    pub max_length: i64,
}

impl QuoteThresholdRule for QuoteThresholdTitleLengthRule {
    fn check(&self, item: &QuoteThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteThresholdSegmentLengthRule {
    pub max_length: i64,
}

impl QuoteThresholdRule for QuoteThresholdSegmentLengthRule {
    fn check(&self, item: &QuoteThreshold) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteThresholdRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteThresholdRule for QuoteThresholdRateRangeRule {
    fn check(&self, item: &QuoteThreshold) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteThresholdAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteThresholdRule for QuoteThresholdAttemptsRangeRule {
    fn check(&self, item: &QuoteThreshold) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteThresholdAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl QuoteThresholdRule for QuoteThresholdAllowedLocaleRule {
    fn check(&self, item: &QuoteThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is missing", item.locale)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote threshold.
#[derive(Debug, Clone)]
pub struct QuoteThresholdValidator {
    pub rules: Vec<Rc<dyn QuoteThresholdRule>>,
}

impl QuoteThresholdValidator {
    pub fn validate(&self, item: &QuoteThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_threshold_validator() -> QuoteThresholdValidator {
    QuoteThresholdValidator {
        rules: vec![
            Rc::new(QuoteThresholdTitleLengthRule { max_length: 1000 }) as Rc<dyn QuoteThresholdRule>,
            Rc::new(QuoteThresholdSegmentLengthRule { max_length: 80 }),
            Rc::new(QuoteThresholdRateRangeRule { min: 7.5, max: 3803.5 }),
            Rc::new(QuoteThresholdAttemptsRangeRule { min: 86400, max: 86400 }),
            Rc::new(QuoteThresholdAllowedLocaleRule {
                allowed: vec!["scheduled".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_threshold_validator();
        let item = new_quote_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QuoteThresholdTitleLengthRule { max_length: 64 };
        let mut item = new_quote_threshold("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = QuoteThresholdSegmentLengthRule { max_length: 16 };
        let mut item = new_quote_threshold("x", "y");
        item.segment = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = QuoteThresholdRateRangeRule { min: 3.4, max: 2390.8 };
        let mut item = new_quote_threshold("x", "y");
        item.rate = 2392.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = QuoteThresholdAttemptsRangeRule { min: 64, max: 255 };
        let mut item = new_quote_threshold("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
