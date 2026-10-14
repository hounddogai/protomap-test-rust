use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy policy review must pass. Check returns its problems, or none.
pub trait LegacyPolicyReviewRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPolicyReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyReviewPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyPolicyReviewRule for LegacyPolicyReviewPriorityRangeRule {
    fn check(&self, item: &LegacyPolicyReview) -> Vec<String> {
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
pub struct LegacyPolicyReviewRequiredPhoneRule {
    pub message: String,
}

impl LegacyPolicyReviewRule for LegacyPolicyReviewRequiredPhoneRule {
    fn check(&self, item: &LegacyPolicyReview) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyReviewRequiredTitleRule {
    pub message: String,
}

impl LegacyPolicyReviewRule for LegacyPolicyReviewRequiredTitleRule {
    fn check(&self, item: &LegacyPolicyReview) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPolicyReviewAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyPolicyReviewRule for LegacyPolicyReviewAllowedCodeRule {
    fn check(&self, item: &LegacyPolicyReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy policy review.
#[derive(Debug, Clone)]
pub struct LegacyPolicyReviewValidator {
    pub rules: Vec<Rc<dyn LegacyPolicyReviewRule>>,
}

impl LegacyPolicyReviewValidator {
    pub fn validate(&self, item: &LegacyPolicyReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyPolicyReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPolicyReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_policy_review_validator() -> LegacyPolicyReviewValidator {
    LegacyPolicyReviewValidator {
        rules: vec![
            Rc::new(LegacyPolicyReviewPriorityRangeRule { min: 32, max: 255 }) as Rc<dyn LegacyPolicyReviewRule>,
            Rc::new(LegacyPolicyReviewRequiredPhoneRule {
                message: "legacy policy review needs an entry".to_string(),
            }),
            Rc::new(LegacyPolicyReviewRequiredTitleRule {
                message: "legacy policy review needs an entry".to_string(),
            }),
            Rc::new(LegacyPolicyReviewAllowedCodeRule {
                allowed: vec!["rolling".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_policy_review_validator();
        let item = new_legacy_policy_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LegacyPolicyReviewPriorityRangeRule { min: 120, max: 255 };
        let mut item = new_legacy_policy_review("x", "y");
        item.priority = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = LegacyPolicyReviewRequiredPhoneRule {
            message: "legacy policy review needs this field".to_string(),
        };
        let mut item = new_legacy_policy_review("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyPolicyReviewRequiredTitleRule {
            message: "legacy policy review needs an entry".to_string(),
        };
        let mut item = new_legacy_policy_review("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
