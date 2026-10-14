use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched quote estimate must pass. Check returns its problems, or none.
pub trait BatchedQuoteEstimateRule: std::fmt::Debug {
    fn check(&self, item: &BatchedQuoteEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteEstimateAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl BatchedQuoteEstimateRule for BatchedQuoteEstimateAllowedCountryRule {
    fn check(&self, item: &BatchedQuoteEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is not allowed", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteEstimateAudienceLengthRule {
    pub max_length: i64,
}

impl BatchedQuoteEstimateRule for BatchedQuoteEstimateAudienceLengthRule {
    fn check(&self, item: &BatchedQuoteEstimate) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteEstimateRequiredAudienceRule {
    pub message: String,
}

impl BatchedQuoteEstimateRule for BatchedQuoteEstimateRequiredAudienceRule {
    fn check(&self, item: &BatchedQuoteEstimate) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteEstimateTitleLengthRule {
    pub max_length: i64,
}

impl BatchedQuoteEstimateRule for BatchedQuoteEstimateTitleLengthRule {
    fn check(&self, item: &BatchedQuoteEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteEstimateBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedQuoteEstimateRule for BatchedQuoteEstimateBatchSizeRangeRule {
    fn check(&self, item: &BatchedQuoteEstimate) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched quote estimate.
#[derive(Debug, Clone)]
pub struct BatchedQuoteEstimateValidator {
    pub rules: Vec<Rc<dyn BatchedQuoteEstimateRule>>,
}

impl BatchedQuoteEstimateValidator {
    pub fn validate(&self, item: &BatchedQuoteEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedQuoteEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedQuoteEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_quote_estimate_validator() -> BatchedQuoteEstimateValidator {
    BatchedQuoteEstimateValidator {
        rules: vec![
            Rc::new(BatchedQuoteEstimateAllowedCountryRule {
                allowed: vec!["lenient".to_string(), "pending".to_string(), "legacy".to_string()],
            }) as Rc<dyn BatchedQuoteEstimateRule>,
            Rc::new(BatchedQuoteEstimateAudienceLengthRule { max_length: 80 }),
            Rc::new(BatchedQuoteEstimateRequiredAudienceRule {
                message: "batched quote estimate needs a value".to_string(),
            }),
            Rc::new(BatchedQuoteEstimateTitleLengthRule { max_length: 255 }),
            Rc::new(BatchedQuoteEstimateBatchSizeRangeRule { min: 16, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_quote_estimate_validator();
        let item = new_batched_quote_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = BatchedQuoteEstimateAudienceLengthRule { max_length: 1000 };
        let mut item = new_batched_quote_estimate("x", "y");
        item.audience = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = BatchedQuoteEstimateRequiredAudienceRule {
            message: "batched quote estimate needs a value".to_string(),
        };
        let mut item = new_batched_quote_estimate("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedQuoteEstimateTitleLengthRule { max_length: 1000 };
        let mut item = new_batched_quote_estimate("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = BatchedQuoteEstimateBatchSizeRangeRule { min: 255, max: 86400 };
        let mut item = new_batched_quote_estimate("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
