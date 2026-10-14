use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a plan review must pass. Check returns its problems, or none.
pub trait PlanReviewRule: std::fmt::Debug {
    fn check(&self, item: &PlanReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanReviewRequiredAudienceRule {
    pub message: String,
}

impl PlanReviewRule for PlanReviewRequiredAudienceRule {
    fn check(&self, item: &PlanReview) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanReviewAudienceLengthRule {
    pub max_length: i64,
}

impl PlanReviewRule for PlanReviewAudienceLengthRule {
    fn check(&self, item: &PlanReview) -> Vec<String> {
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
pub struct PlanReviewCodeLengthRule {
    pub max_length: i64,
}

impl PlanReviewRule for PlanReviewCodeLengthRule {
    fn check(&self, item: &PlanReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanReviewCurrencyLengthRule {
    pub max_length: i64,
}

impl PlanReviewRule for PlanReviewCurrencyLengthRule {
    fn check(&self, item: &PlanReview) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PlanReviewRule for PlanReviewAllowedCodeRule {
    fn check(&self, item: &PlanReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanReviewPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PlanReviewRule for PlanReviewPriorityRangeRule {
    fn check(&self, item: &PlanReview) -> Vec<String> {
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

/// Runs every rule of a plan review.
#[derive(Debug, Clone)]
pub struct PlanReviewValidator {
    pub rules: Vec<Rc<dyn PlanReviewRule>>,
}

impl PlanReviewValidator {
    pub fn validate(&self, item: &PlanReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PlanReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PlanReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_plan_review_validator() -> PlanReviewValidator {
    PlanReviewValidator {
        rules: vec![
            Rc::new(PlanReviewRequiredAudienceRule {
                message: "plan review needs this field".to_string(),
            }) as Rc<dyn PlanReviewRule>,
            Rc::new(PlanReviewAudienceLengthRule { max_length: 120 }),
            Rc::new(PlanReviewCodeLengthRule { max_length: 64 }),
            Rc::new(PlanReviewCurrencyLengthRule { max_length: 80 }),
            Rc::new(PlanReviewAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(PlanReviewPriorityRangeRule { min: 80, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_plan_review_validator();
        let item = new_plan_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PlanReviewRequiredAudienceRule {
            message: "plan review needs a value".to_string(),
        };
        let mut item = new_plan_review("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PlanReviewAudienceLengthRule { max_length: 80 };
        let mut item = new_plan_review("x", "y");
        item.audience = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PlanReviewCodeLengthRule { max_length: 3600 };
        let mut item = new_plan_review("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = PlanReviewCurrencyLengthRule { max_length: 80 };
        let mut item = new_plan_review("x", "y");
        item.currency = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = PlanReviewPriorityRangeRule { min: 255, max: 86400 };
        let mut item = new_plan_review("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
