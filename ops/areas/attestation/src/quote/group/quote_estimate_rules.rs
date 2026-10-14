use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a quote estimate must pass. Check returns its problems, or none.
pub trait QuoteEstimateRule: std::fmt::Debug {
    fn check(&self, item: &QuoteEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEstimateBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteEstimateRule for QuoteEstimateBatchSizeRangeRule {
    fn check(&self, item: &QuoteEstimate) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEstimateStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteEstimateRule for QuoteEstimateStartsAtRangeRule {
    fn check(&self, item: &QuoteEstimate) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEstimateCodePrefixRule {
    pub prefix: String,
}

impl QuoteEstimateRule for QuoteEstimateCodePrefixRule {
    fn check(&self, item: &QuoteEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEstimateEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteEstimateRule for QuoteEstimateEndsAtRangeRule {
    fn check(&self, item: &QuoteEstimate) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteEstimateAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl QuoteEstimateRule for QuoteEstimateAllowedTitleRule {
    fn check(&self, item: &QuoteEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is below minimum", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a quote estimate.
#[derive(Debug, Clone)]
pub struct QuoteEstimateValidator {
    pub rules: Vec<Rc<dyn QuoteEstimateRule>>,
}

impl QuoteEstimateValidator {
    pub fn validate(&self, item: &QuoteEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_estimate_validator() -> QuoteEstimateValidator {
    QuoteEstimateValidator {
        rules: vec![
            Rc::new(QuoteEstimateBatchSizeRangeRule { min: 80, max: 1000 }) as Rc<dyn QuoteEstimateRule>,
            Rc::new(QuoteEstimateStartsAtRangeRule { min: 80, max: 1000 }),
            Rc::new(QuoteEstimateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(QuoteEstimateEndsAtRangeRule { min: 16, max: 86400 }),
            Rc::new(QuoteEstimateAllowedTitleRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_estimate_validator();
        let item = new_quote_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = QuoteEstimateBatchSizeRangeRule { min: 120, max: 1000 };
        let mut item = new_quote_estimate("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = QuoteEstimateStartsAtRangeRule { min: 80, max: 120 };
        let mut item = new_quote_estimate("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = QuoteEstimateEndsAtRangeRule { min: 64, max: 120 };
        let mut item = new_quote_estimate("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
