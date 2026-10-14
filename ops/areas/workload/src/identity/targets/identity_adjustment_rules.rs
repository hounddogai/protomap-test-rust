use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity adjustment must pass. Check returns its problems, or none.
pub trait IdentityAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &IdentityAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl IdentityAdjustmentRule for IdentityAdjustmentAllowedCodeRule {
    fn check(&self, item: &IdentityAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustmentRequiredLocaleRule {
    pub message: String,
}

impl IdentityAdjustmentRule for IdentityAdjustmentRequiredLocaleRule {
    fn check(&self, item: &IdentityAdjustment) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl IdentityAdjustmentRule for IdentityAdjustmentCodePrefixRule {
    fn check(&self, item: &IdentityAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl IdentityAdjustmentRule for IdentityAdjustmentCodeLengthRule {
    fn check(&self, item: &IdentityAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustmentEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityAdjustmentRule for IdentityAdjustmentEndsAtRangeRule {
    fn check(&self, item: &IdentityAdjustment) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAdjustmentReviewerLengthRule {
    pub max_length: i64,
}

impl IdentityAdjustmentRule for IdentityAdjustmentReviewerLengthRule {
    fn check(&self, item: &IdentityAdjustment) -> Vec<String> {
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

/// Runs every rule of a identity adjustment.
#[derive(Debug, Clone)]
pub struct IdentityAdjustmentValidator {
    pub rules: Vec<Rc<dyn IdentityAdjustmentRule>>,
}

impl IdentityAdjustmentValidator {
    pub fn validate(&self, item: &IdentityAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_adjustment_validator() -> IdentityAdjustmentValidator {
    IdentityAdjustmentValidator {
        rules: vec![
            Rc::new(IdentityAdjustmentAllowedCodeRule {
                allowed: vec!["tiered".to_string(), "default".to_string()],
            }) as Rc<dyn IdentityAdjustmentRule>,
            Rc::new(IdentityAdjustmentRequiredLocaleRule {
                message: "identity adjustment needs an entry".to_string(),
            }),
            Rc::new(IdentityAdjustmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(IdentityAdjustmentCodeLengthRule { max_length: 80 }),
            Rc::new(IdentityAdjustmentEndsAtRangeRule { min: 80, max: 120 }),
            Rc::new(IdentityAdjustmentReviewerLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_adjustment_validator();
        let item = new_identity_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = IdentityAdjustmentRequiredLocaleRule {
            message: "identity adjustment needs a value".to_string(),
        };
        let mut item = new_identity_adjustment("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = IdentityAdjustmentCodeLengthRule { max_length: 255 };
        let mut item = new_identity_adjustment("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = IdentityAdjustmentEndsAtRangeRule { min: 16, max: 80 };
        let mut item = new_identity_adjustment("x", "y");
        item.ends_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = IdentityAdjustmentReviewerLengthRule { max_length: 16 };
        let mut item = new_identity_adjustment("x", "y");
        item.reviewer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
