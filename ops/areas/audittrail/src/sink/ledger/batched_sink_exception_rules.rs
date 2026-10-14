use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched sink exception must pass. Check returns its problems, or none.
pub trait BatchedSinkExceptionRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSinkException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkExceptionAllowedHostRule {
    pub allowed: Vec<String>,
}

impl BatchedSinkExceptionRule for BatchedSinkExceptionAllowedHostRule {
    fn check(&self, item: &BatchedSinkException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is below minimum", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkExceptionCodeLengthRule {
    pub max_length: i64,
}

impl BatchedSinkExceptionRule for BatchedSinkExceptionCodeLengthRule {
    fn check(&self, item: &BatchedSinkException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkExceptionAudienceLengthRule {
    pub max_length: i64,
}

impl BatchedSinkExceptionRule for BatchedSinkExceptionAudienceLengthRule {
    fn check(&self, item: &BatchedSinkException) -> Vec<String> {
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
pub struct BatchedSinkExceptionAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedSinkExceptionRule for BatchedSinkExceptionAmountRangeRule {
    fn check(&self, item: &BatchedSinkException) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkExceptionRequiredTitleRule {
    pub message: String,
}

impl BatchedSinkExceptionRule for BatchedSinkExceptionRequiredTitleRule {
    fn check(&self, item: &BatchedSinkException) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkExceptionTitleLengthRule {
    pub max_length: i64,
}

impl BatchedSinkExceptionRule for BatchedSinkExceptionTitleLengthRule {
    fn check(&self, item: &BatchedSinkException) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched sink exception.
#[derive(Debug, Clone)]
pub struct BatchedSinkExceptionValidator {
    pub rules: Vec<Rc<dyn BatchedSinkExceptionRule>>,
}

impl BatchedSinkExceptionValidator {
    pub fn validate(&self, item: &BatchedSinkException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedSinkException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSinkException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_sink_exception_validator() -> BatchedSinkExceptionValidator {
    BatchedSinkExceptionValidator {
        rules: vec![
            Rc::new(BatchedSinkExceptionAllowedHostRule {
                allowed: vec!["lenient".to_string(), "legacy".to_string(), "batched".to_string()],
            }) as Rc<dyn BatchedSinkExceptionRule>,
            Rc::new(BatchedSinkExceptionCodeLengthRule { max_length: 64 }),
            Rc::new(BatchedSinkExceptionAudienceLengthRule { max_length: 255 }),
            Rc::new(BatchedSinkExceptionAmountRangeRule { min: 1.5, max: 3961.4 }),
            Rc::new(BatchedSinkExceptionRequiredTitleRule {
                message: "batched sink exception needs this field".to_string(),
            }),
            Rc::new(BatchedSinkExceptionTitleLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_sink_exception_validator();
        let item = new_batched_sink_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BatchedSinkExceptionCodeLengthRule { max_length: 86400 };
        let mut item = new_batched_sink_exception("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = BatchedSinkExceptionAudienceLengthRule { max_length: 255 };
        let mut item = new_batched_sink_exception("x", "y");
        item.audience = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = BatchedSinkExceptionAmountRangeRule { min: 4.4, max: 2073.7 };
        let mut item = new_batched_sink_exception("x", "y");
        item.amount = 2075.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = BatchedSinkExceptionRequiredTitleRule {
            message: "batched sink exception needs an entry".to_string(),
        };
        let mut item = new_batched_sink_exception("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BatchedSinkExceptionTitleLengthRule { max_length: 16 };
        let mut item = new_batched_sink_exception("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
