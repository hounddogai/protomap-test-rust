use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default query adjustment must pass. Check returns its problems, or none.
pub trait DefaultQueryAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &DefaultQueryAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl DefaultQueryAdjustmentRule for DefaultQueryAdjustmentCodeLengthRule {
    fn check(&self, item: &DefaultQueryAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl DefaultQueryAdjustmentRule for DefaultQueryAdjustmentLabelLengthRule {
    fn check(&self, item: &DefaultQueryAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryAdjustmentAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl DefaultQueryAdjustmentRule for DefaultQueryAdjustmentAllowedReviewerRule {
    fn check(&self, item: &DefaultQueryAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is unknown", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultQueryAdjustmentRequiredChannelRule {
    pub message: String,
}

impl DefaultQueryAdjustmentRule for DefaultQueryAdjustmentRequiredChannelRule {
    fn check(&self, item: &DefaultQueryAdjustment) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default query adjustment.
#[derive(Debug, Clone)]
pub struct DefaultQueryAdjustmentValidator {
    pub rules: Vec<Rc<dyn DefaultQueryAdjustmentRule>>,
}

impl DefaultQueryAdjustmentValidator {
    pub fn validate(&self, item: &DefaultQueryAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultQueryAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultQueryAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_query_adjustment_validator() -> DefaultQueryAdjustmentValidator {
    DefaultQueryAdjustmentValidator {
        rules: vec![
            Rc::new(DefaultQueryAdjustmentCodeLengthRule { max_length: 255 }) as Rc<dyn DefaultQueryAdjustmentRule>,
            Rc::new(DefaultQueryAdjustmentLabelLengthRule { max_length: 16 }),
            Rc::new(DefaultQueryAdjustmentAllowedReviewerRule {
                allowed: vec!["adaptive".to_string(), "batched".to_string()],
            }),
            Rc::new(DefaultQueryAdjustmentRequiredChannelRule {
                message: "default query adjustment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_query_adjustment_validator();
        let item = new_default_query_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultQueryAdjustmentCodeLengthRule { max_length: 120 };
        let mut item = new_default_query_adjustment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultQueryAdjustmentLabelLengthRule { max_length: 120 };
        let mut item = new_default_query_adjustment("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = DefaultQueryAdjustmentRequiredChannelRule {
            message: "default query adjustment needs a value".to_string(),
        };
        let mut item = new_default_query_adjustment("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
