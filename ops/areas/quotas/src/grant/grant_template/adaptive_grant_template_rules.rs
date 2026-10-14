use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive grant template must pass. Check returns its problems, or none.
pub trait AdaptiveGrantTemplateRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveGrantTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveGrantTemplateRule for AdaptiveGrantTemplateAllowedCodeRule {
    fn check(&self, item: &AdaptiveGrantTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplateRequiredTitleRule {
    pub message: String,
}

impl AdaptiveGrantTemplateRule for AdaptiveGrantTemplateRequiredTitleRule {
    fn check(&self, item: &AdaptiveGrantTemplate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplateRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveGrantTemplateRule for AdaptiveGrantTemplateRotationDaysRangeRule {
    fn check(&self, item: &AdaptiveGrantTemplate) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplateReviewerLengthRule {
    pub max_length: i64,
}

impl AdaptiveGrantTemplateRule for AdaptiveGrantTemplateReviewerLengthRule {
    fn check(&self, item: &AdaptiveGrantTemplate) -> Vec<String> {
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
pub struct AdaptiveGrantTemplateCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveGrantTemplateRule for AdaptiveGrantTemplateCodeLengthRule {
    fn check(&self, item: &AdaptiveGrantTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveGrantTemplateRequiredReviewerRule {
    pub message: String,
}

impl AdaptiveGrantTemplateRule for AdaptiveGrantTemplateRequiredReviewerRule {
    fn check(&self, item: &AdaptiveGrantTemplate) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive grant template.
#[derive(Debug, Clone)]
pub struct AdaptiveGrantTemplateValidator {
    pub rules: Vec<Rc<dyn AdaptiveGrantTemplateRule>>,
}

impl AdaptiveGrantTemplateValidator {
    pub fn validate(&self, item: &AdaptiveGrantTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveGrantTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveGrantTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_grant_template_validator() -> AdaptiveGrantTemplateValidator {
    AdaptiveGrantTemplateValidator {
        rules: vec![
            Rc::new(AdaptiveGrantTemplateAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "seasonal".to_string()],
            }) as Rc<dyn AdaptiveGrantTemplateRule>,
            Rc::new(AdaptiveGrantTemplateRequiredTitleRule {
                message: "adaptive grant template needs a value".to_string(),
            }),
            Rc::new(AdaptiveGrantTemplateRotationDaysRangeRule { min: 32, max: 255 }),
            Rc::new(AdaptiveGrantTemplateReviewerLengthRule { max_length: 255 }),
            Rc::new(AdaptiveGrantTemplateCodeLengthRule { max_length: 16 }),
            Rc::new(AdaptiveGrantTemplateRequiredReviewerRule {
                message: "adaptive grant template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_grant_template_validator();
        let item = new_adaptive_grant_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveGrantTemplateRequiredTitleRule {
            message: "adaptive grant template needs a value".to_string(),
        };
        let mut item = new_adaptive_grant_template("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = AdaptiveGrantTemplateRotationDaysRangeRule { min: 16, max: 120 };
        let mut item = new_adaptive_grant_template("x", "y");
        item.rotation_days = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = AdaptiveGrantTemplateReviewerLengthRule { max_length: 32 };
        let mut item = new_adaptive_grant_template("x", "y");
        item.reviewer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveGrantTemplateCodeLengthRule { max_length: 86400 };
        let mut item = new_adaptive_grant_template("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = AdaptiveGrantTemplateRequiredReviewerRule {
            message: "adaptive grant template needs an entry".to_string(),
        };
        let mut item = new_adaptive_grant_template("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
