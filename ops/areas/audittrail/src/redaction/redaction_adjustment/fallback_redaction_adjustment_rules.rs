use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback redaction adjustment must pass. Check returns its problems, or none.
pub trait FallbackRedactionAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRedactionAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl FallbackRedactionAdjustmentRule for FallbackRedactionAdjustmentTitleLengthRule {
    fn check(&self, item: &FallbackRedactionAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustmentAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl FallbackRedactionAdjustmentRule for FallbackRedactionAdjustmentAllowedLocaleRule {
    fn check(&self, item: &FallbackRedactionAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is not allowed", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustmentRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRedactionAdjustmentRule for FallbackRedactionAdjustmentRevisionRangeRule {
    fn check(&self, item: &FallbackRedactionAdjustment) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl FallbackRedactionAdjustmentRule for FallbackRedactionAdjustmentCodePrefixRule {
    fn check(&self, item: &FallbackRedactionAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackRedactionAdjustmentRule for FallbackRedactionAdjustmentPriorityRangeRule {
    fn check(&self, item: &FallbackRedactionAdjustment) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionAdjustmentRequiredDescriptionRule {
    pub message: String,
}

impl FallbackRedactionAdjustmentRule for FallbackRedactionAdjustmentRequiredDescriptionRule {
    fn check(&self, item: &FallbackRedactionAdjustment) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback redaction adjustment.
#[derive(Debug, Clone)]
pub struct FallbackRedactionAdjustmentValidator {
    pub rules: Vec<Rc<dyn FallbackRedactionAdjustmentRule>>,
}

impl FallbackRedactionAdjustmentValidator {
    pub fn validate(&self, item: &FallbackRedactionAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRedactionAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRedactionAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_redaction_adjustment_validator() -> FallbackRedactionAdjustmentValidator {
    FallbackRedactionAdjustmentValidator {
        rules: vec![
            Rc::new(FallbackRedactionAdjustmentTitleLengthRule { max_length: 32 })
                as Rc<dyn FallbackRedactionAdjustmentRule>,
            Rc::new(FallbackRedactionAdjustmentAllowedLocaleRule { allowed: Vec::new() }),
            Rc::new(FallbackRedactionAdjustmentRevisionRangeRule { min: 64, max: 120 }),
            Rc::new(FallbackRedactionAdjustmentCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(FallbackRedactionAdjustmentPriorityRangeRule { min: 120, max: 3600 }),
            Rc::new(FallbackRedactionAdjustmentRequiredDescriptionRule {
                message: "fallback redaction adjustment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_redaction_adjustment_validator();
        let item = new_fallback_redaction_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackRedactionAdjustmentTitleLengthRule { max_length: 64 };
        let mut item = new_fallback_redaction_adjustment("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = FallbackRedactionAdjustmentRevisionRangeRule { min: 32, max: 86400 };
        let mut item = new_fallback_redaction_adjustment("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FallbackRedactionAdjustmentPriorityRangeRule { min: 120, max: 3600 };
        let mut item = new_fallback_redaction_adjustment("x", "y");
        item.priority = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = FallbackRedactionAdjustmentRequiredDescriptionRule {
            message: "fallback redaction adjustment needs this field".to_string(),
        };
        let mut item = new_fallback_redaction_adjustment("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
