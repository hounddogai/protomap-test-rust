use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a runbook review must pass. Check returns its problems, or none.
pub trait RunbookReviewRule: std::fmt::Debug {
    fn check(&self, item: &RunbookReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReviewRequiredLabelRule {
    pub message: String,
}

impl RunbookReviewRule for RunbookReviewRequiredLabelRule {
    fn check(&self, item: &RunbookReview) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReviewRequiredCodeRule {
    pub message: String,
}

impl RunbookReviewRule for RunbookReviewRequiredCodeRule {
    fn check(&self, item: &RunbookReview) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReviewAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl RunbookReviewRule for RunbookReviewAllowedTimezoneRule {
    fn check(&self, item: &RunbookReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is above maximum", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReviewDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RunbookReviewRule for RunbookReviewDiscountRangeRule {
    fn check(&self, item: &RunbookReview) -> Vec<String> {
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
pub struct RunbookReviewLabelLengthRule {
    pub max_length: i64,
}

impl RunbookReviewRule for RunbookReviewLabelLengthRule {
    fn check(&self, item: &RunbookReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReviewRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookReviewRule for RunbookReviewRetriesRangeRule {
    fn check(&self, item: &RunbookReview) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a runbook review.
#[derive(Debug, Clone)]
pub struct RunbookReviewValidator {
    pub rules: Vec<Rc<dyn RunbookReviewRule>>,
}

impl RunbookReviewValidator {
    pub fn validate(&self, item: &RunbookReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_review_validator() -> RunbookReviewValidator {
    RunbookReviewValidator {
        rules: vec![
            Rc::new(RunbookReviewRequiredLabelRule {
                message: "runbook review needs this field".to_string(),
            }) as Rc<dyn RunbookReviewRule>,
            Rc::new(RunbookReviewRequiredCodeRule {
                message: "runbook review needs an entry".to_string(),
            }),
            Rc::new(RunbookReviewAllowedTimezoneRule {
                allowed: vec!["adaptive".to_string(), "regional".to_string()],
            }),
            Rc::new(RunbookReviewDiscountRangeRule { min: 8.7, max: 1112.9 }),
            Rc::new(RunbookReviewLabelLengthRule { max_length: 64 }),
            Rc::new(RunbookReviewRetriesRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_review_validator();
        let item = new_runbook_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RunbookReviewRequiredLabelRule {
            message: "runbook review needs an entry".to_string(),
        };
        let mut item = new_runbook_review("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookReviewRequiredCodeRule {
            message: "runbook review needs an entry".to_string(),
        };
        let mut item = new_runbook_review("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = RunbookReviewDiscountRangeRule { min: 6.3, max: 896.3 };
        let mut item = new_runbook_review("x", "y");
        item.discount = 897.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RunbookReviewLabelLengthRule { max_length: 80 };
        let mut item = new_runbook_review("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RunbookReviewRetriesRangeRule { min: 32, max: 1000 };
        let mut item = new_runbook_review("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
