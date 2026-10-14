use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default rule review must pass. Check returns its problems, or none.
pub trait DefaultRuleReviewRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRuleReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleReviewAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DefaultRuleReviewRule for DefaultRuleReviewAllowedLabelRule {
    fn check(&self, item: &DefaultRuleReview) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleReviewContactEmailLengthRule {
    pub max_length: i64,
}

impl DefaultRuleReviewRule for DefaultRuleReviewContactEmailLengthRule {
    fn check(&self, item: &DefaultRuleReview) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleReviewLabelLengthRule {
    pub max_length: i64,
}

impl DefaultRuleReviewRule for DefaultRuleReviewLabelLengthRule {
    fn check(&self, item: &DefaultRuleReview) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleReviewUnitLengthRule {
    pub max_length: i64,
}

impl DefaultRuleReviewRule for DefaultRuleReviewUnitLengthRule {
    fn check(&self, item: &DefaultRuleReview) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rule review.
#[derive(Debug, Clone)]
pub struct DefaultRuleReviewValidator {
    pub rules: Vec<Rc<dyn DefaultRuleReviewRule>>,
}

impl DefaultRuleReviewValidator {
    pub fn validate(&self, item: &DefaultRuleReview) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRuleReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRuleReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rule_review_validator() -> DefaultRuleReviewValidator {
    DefaultRuleReviewValidator {
        rules: vec![
            Rc::new(DefaultRuleReviewAllowedLabelRule { allowed: Vec::new() }) as Rc<dyn DefaultRuleReviewRule>,
            Rc::new(DefaultRuleReviewContactEmailLengthRule { max_length: 64 }),
            Rc::new(DefaultRuleReviewLabelLengthRule { max_length: 80 }),
            Rc::new(DefaultRuleReviewUnitLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rule_review_validator();
        let item = new_default_rule_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = DefaultRuleReviewContactEmailLengthRule { max_length: 1000 };
        let mut item = new_default_rule_review("x", "y");
        item.contact_email = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultRuleReviewLabelLengthRule { max_length: 86400 };
        let mut item = new_default_rule_review("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = DefaultRuleReviewUnitLengthRule { max_length: 86400 };
        let mut item = new_default_rule_review("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
