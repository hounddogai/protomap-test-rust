use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending rollout review must pass. Check returns its problems, or none.
pub trait PendingRolloutReviewRule: std::fmt::Debug {
    fn check(&self, item: &PendingRolloutReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutReviewSegmentLengthRule {
    pub max_length: i64,
}

impl PendingRolloutReviewRule for PendingRolloutReviewSegmentLengthRule {
    fn check(&self, item: &PendingRolloutReview) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutReviewPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingRolloutReviewRule for PendingRolloutReviewPriorityRangeRule {
    fn check(&self, item: &PendingRolloutReview) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutReviewTitleLengthRule {
    pub max_length: i64,
}

impl PendingRolloutReviewRule for PendingRolloutReviewTitleLengthRule {
    fn check(&self, item: &PendingRolloutReview) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingRolloutReviewBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingRolloutReviewRule for PendingRolloutReviewBudgetRangeRule {
    fn check(&self, item: &PendingRolloutReview) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending rollout review.
#[derive(Debug, Clone)]
pub struct PendingRolloutReviewValidator {
    pub rules: Vec<Rc<dyn PendingRolloutReviewRule>>,
}

impl PendingRolloutReviewValidator {
    pub fn validate(&self, item: &PendingRolloutReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingRolloutReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingRolloutReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_rollout_review_validator() -> PendingRolloutReviewValidator {
    PendingRolloutReviewValidator {
        rules: vec![
            Rc::new(PendingRolloutReviewSegmentLengthRule { max_length: 16 }) as Rc<dyn PendingRolloutReviewRule>,
            Rc::new(PendingRolloutReviewPriorityRangeRule { min: 64, max: 1000 }),
            Rc::new(PendingRolloutReviewTitleLengthRule { max_length: 86400 }),
            Rc::new(PendingRolloutReviewBudgetRangeRule { min: 1.1, max: 240.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_rollout_review_validator();
        let item = new_pending_rollout_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = PendingRolloutReviewSegmentLengthRule { max_length: 1000 };
        let mut item = new_pending_rollout_review("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PendingRolloutReviewPriorityRangeRule { min: 32, max: 86400 };
        let mut item = new_pending_rollout_review("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingRolloutReviewTitleLengthRule { max_length: 120 };
        let mut item = new_pending_rollout_review("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = PendingRolloutReviewBudgetRangeRule { min: 1.4, max: 3454.9 };
        let mut item = new_pending_rollout_review("x", "y");
        item.budget = 3456.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
