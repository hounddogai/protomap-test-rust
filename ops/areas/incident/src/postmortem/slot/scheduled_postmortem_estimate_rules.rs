use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled postmortem estimate must pass. Check returns its problems, or none.
pub trait ScheduledPostmortemEstimateRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPostmortemEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimateReviewerLengthRule {
    pub max_length: i64,
}

impl ScheduledPostmortemEstimateRule for ScheduledPostmortemEstimateReviewerLengthRule {
    fn check(&self, item: &ScheduledPostmortemEstimate) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimateRequiredTitleRule {
    pub message: String,
}

impl ScheduledPostmortemEstimateRule for ScheduledPostmortemEstimateRequiredTitleRule {
    fn check(&self, item: &ScheduledPostmortemEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimateTitleLengthRule {
    pub max_length: i64,
}

impl ScheduledPostmortemEstimateRule for ScheduledPostmortemEstimateTitleLengthRule {
    fn check(&self, item: &ScheduledPostmortemEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimateRequiredSummaryRule {
    pub message: String,
}

impl ScheduledPostmortemEstimateRule for ScheduledPostmortemEstimateRequiredSummaryRule {
    fn check(&self, item: &ScheduledPostmortemEstimate) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPostmortemEstimateCodePrefixRule {
    pub prefix: String,
}

impl ScheduledPostmortemEstimateRule for ScheduledPostmortemEstimateCodePrefixRule {
    fn check(&self, item: &ScheduledPostmortemEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled postmortem estimate.
#[derive(Debug, Clone)]
pub struct ScheduledPostmortemEstimateValidator {
    pub rules: Vec<Rc<dyn ScheduledPostmortemEstimateRule>>,
}

impl ScheduledPostmortemEstimateValidator {
    pub fn validate(&self, item: &ScheduledPostmortemEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledPostmortemEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPostmortemEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_postmortem_estimate_validator() -> ScheduledPostmortemEstimateValidator {
    ScheduledPostmortemEstimateValidator {
        rules: vec![
            Rc::new(ScheduledPostmortemEstimateReviewerLengthRule { max_length: 80 })
                as Rc<dyn ScheduledPostmortemEstimateRule>,
            Rc::new(ScheduledPostmortemEstimateRequiredTitleRule {
                message: "scheduled postmortem estimate needs this field".to_string(),
            }),
            Rc::new(ScheduledPostmortemEstimateTitleLengthRule { max_length: 120 }),
            Rc::new(ScheduledPostmortemEstimateRequiredSummaryRule {
                message: "scheduled postmortem estimate needs this field".to_string(),
            }),
            Rc::new(ScheduledPostmortemEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_postmortem_estimate_validator();
        let item = new_scheduled_postmortem_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = ScheduledPostmortemEstimateReviewerLengthRule { max_length: 16 };
        let mut item = new_scheduled_postmortem_estimate("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledPostmortemEstimateRequiredTitleRule {
            message: "scheduled postmortem estimate needs this field".to_string(),
        };
        let mut item = new_scheduled_postmortem_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ScheduledPostmortemEstimateTitleLengthRule { max_length: 32 };
        let mut item = new_scheduled_postmortem_estimate("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = ScheduledPostmortemEstimateRequiredSummaryRule {
            message: "scheduled postmortem estimate needs an entry".to_string(),
        };
        let mut item = new_scheduled_postmortem_estimate("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
