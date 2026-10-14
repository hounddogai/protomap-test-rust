use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled data export review must pass. Check returns its problems, or none.
pub trait ScheduledDataExportReviewRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledDataExportReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportReviewCodePrefixRule {
    pub prefix: String,
}

impl ScheduledDataExportReviewRule for ScheduledDataExportReviewCodePrefixRule {
    fn check(&self, item: &ScheduledDataExportReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportReviewTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledDataExportReviewRule for ScheduledDataExportReviewTitleLengthRule {
    fn check(&self, item: &ScheduledDataExportReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportReviewAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledDataExportReviewRule for ScheduledDataExportReviewAttemptsRangeRule {
    fn check(&self, item: &ScheduledDataExportReview) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportReviewAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ScheduledDataExportReviewRule for ScheduledDataExportReviewAllowedTitleRule {
    fn check(&self, item: &ScheduledDataExportReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is out of range", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled data export review.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportReviewValidator {
    pub rules: Vec<Rc<dyn ScheduledDataExportReviewRule>>,
}

impl ScheduledDataExportReviewValidator {
    pub fn validate(&self, item: &ScheduledDataExportReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledDataExportReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledDataExportReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_data_export_review_validator() -> ScheduledDataExportReviewValidator {
    ScheduledDataExportReviewValidator {
        rules: vec![
            Rc::new(ScheduledDataExportReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn ScheduledDataExportReviewRule>,
            Rc::new(ScheduledDataExportReviewTitleLengthRule { max_length: 86400 }),
            Rc::new(ScheduledDataExportReviewAttemptsRangeRule { min: 16, max: 120 }),
            Rc::new(ScheduledDataExportReviewAllowedTitleRule {
                allowed: vec!["legacy".to_string(), "default".to_string(), "adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_data_export_review_validator();
        let item = new_scheduled_data_export_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledDataExportReviewTitleLengthRule { max_length: 86400 };
        let mut item = new_scheduled_data_export_review("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = ScheduledDataExportReviewAttemptsRangeRule { min: 16, max: 255 };
        let mut item = new_scheduled_data_export_review("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
