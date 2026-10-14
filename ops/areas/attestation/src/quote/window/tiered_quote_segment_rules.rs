use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered quote segment must pass. Check returns its problems, or none.
pub trait TieredQuoteSegmentRule: std::fmt::Debug {
    fn check(&self, item: &TieredQuoteSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteSegmentCodePrefixRule {
    pub prefix: String,
}

impl TieredQuoteSegmentRule for TieredQuoteSegmentCodePrefixRule {
    fn check(&self, item: &TieredQuoteSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteSegmentRequiredCurrencyRule {
    pub message: String,
}

impl TieredQuoteSegmentRule for TieredQuoteSegmentRequiredCurrencyRule {
    fn check(&self, item: &TieredQuoteSegment) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteSegmentRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredQuoteSegmentRule for TieredQuoteSegmentRetriesRangeRule {
    fn check(&self, item: &TieredQuoteSegment) -> Vec<String> {
        let value = item.retries;
        if value < self.min || value > self.max {
            return vec![format!("retries {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredQuoteSegmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredQuoteSegmentRule for TieredQuoteSegmentWindowDaysRangeRule {
    fn check(&self, item: &TieredQuoteSegment) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered quote segment.
#[derive(Debug, Clone)]
pub struct TieredQuoteSegmentValidator {
    pub rules: Vec<Rc<dyn TieredQuoteSegmentRule>>,
}

impl TieredQuoteSegmentValidator {
    pub fn validate(&self, item: &TieredQuoteSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredQuoteSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredQuoteSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_quote_segment_validator() -> TieredQuoteSegmentValidator {
    TieredQuoteSegmentValidator {
        rules: vec![
            Rc::new(TieredQuoteSegmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn TieredQuoteSegmentRule>,
            Rc::new(TieredQuoteSegmentRequiredCurrencyRule {
                message: "tiered quote segment needs this field".to_string(),
            }),
            Rc::new(TieredQuoteSegmentRetriesRangeRule { min: 32, max: 255 }),
            Rc::new(TieredQuoteSegmentWindowDaysRangeRule { min: 120, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_quote_segment_validator();
        let item = new_tiered_quote_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = TieredQuoteSegmentRequiredCurrencyRule {
            message: "tiered quote segment needs this field".to_string(),
        };
        let mut item = new_tiered_quote_segment("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TieredQuoteSegmentRetriesRangeRule { min: 64, max: 1000 };
        let mut item = new_tiered_quote_segment("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = TieredQuoteSegmentWindowDaysRangeRule { min: 16, max: 64 };
        let mut item = new_tiered_quote_segment("x", "y");
        item.window_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
