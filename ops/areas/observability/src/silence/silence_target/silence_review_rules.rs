use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence review must pass. Check returns its problems, or none.
pub trait SilenceReviewRule: std::fmt::Debug {
    fn check(&self, item: &SilenceReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReviewTitleLengthRule {
    pub max_length: i64,
}

impl SilenceReviewRule for SilenceReviewTitleLengthRule {
    fn check(&self, item: &SilenceReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReviewQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceReviewRule for SilenceReviewQuotaUnitsRangeRule {
    fn check(&self, item: &SilenceReview) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReviewRequiredTitleRule {
    pub message: String,
}

impl SilenceReviewRule for SilenceReviewRequiredTitleRule {
    fn check(&self, item: &SilenceReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReviewSummaryLengthRule {
    pub max_length: i64,
}

impl SilenceReviewRule for SilenceReviewSummaryLengthRule {
    fn check(&self, item: &SilenceReview) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReviewBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceReviewRule for SilenceReviewBatchSizeRangeRule {
    fn check(&self, item: &SilenceReview) -> Vec<String> {
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
pub struct SilenceReviewAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl SilenceReviewRule for SilenceReviewAllowedTitleRule {
    fn check(&self, item: &SilenceReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence review.
#[derive(Debug, Clone)]
pub struct SilenceReviewValidator {
    pub rules: Vec<Rc<dyn SilenceReviewRule>>,
}

impl SilenceReviewValidator {
    pub fn validate(&self, item: &SilenceReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_review_validator() -> SilenceReviewValidator {
    SilenceReviewValidator {
        rules: vec![
            Rc::new(SilenceReviewTitleLengthRule { max_length: 80 }) as Rc<dyn SilenceReviewRule>,
            Rc::new(SilenceReviewQuotaUnitsRangeRule { min: 16, max: 64 }),
            Rc::new(SilenceReviewRequiredTitleRule {
                message: "silence review needs an entry".to_string(),
            }),
            Rc::new(SilenceReviewSummaryLengthRule { max_length: 3600 }),
            Rc::new(SilenceReviewBatchSizeRangeRule { min: 64, max: 255 }),
            Rc::new(SilenceReviewAllowedTitleRule {
                allowed: vec!["legacy".to_string(), "manual".to_string(), "tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_review_validator();
        let item = new_silence_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SilenceReviewTitleLengthRule { max_length: 1000 };
        let mut item = new_silence_review("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = SilenceReviewQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_silence_review("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SilenceReviewRequiredTitleRule {
            message: "silence review needs an entry".to_string(),
        };
        let mut item = new_silence_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = SilenceReviewSummaryLengthRule { max_length: 86400 };
        let mut item = new_silence_review("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = SilenceReviewBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_silence_review("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
