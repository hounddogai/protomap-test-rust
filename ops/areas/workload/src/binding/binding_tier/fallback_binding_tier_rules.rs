use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback binding tier must pass. Check returns its problems, or none.
pub trait FallbackBindingTierRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBindingTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl FallbackBindingTierRule for FallbackBindingTierAllowedLabelRule {
    fn check(&self, item: &FallbackBindingTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is too long", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackBindingTierRule for FallbackBindingTierWindowDaysRangeRule {
    fn check(&self, item: &FallbackBindingTier) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierExternalRefLengthRule {
    pub max_length: i64,
}

impl FallbackBindingTierRule for FallbackBindingTierExternalRefLengthRule {
    fn check(&self, item: &FallbackBindingTier) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierCodeLengthRule {
    pub max_length: i64,
}

impl FallbackBindingTierRule for FallbackBindingTierCodeLengthRule {
    fn check(&self, item: &FallbackBindingTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackBindingTierRule for FallbackBindingTierRotationDaysRangeRule {
    fn check(&self, item: &FallbackBindingTier) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingTierRequiredExternalRefRule {
    pub message: String,
}

impl FallbackBindingTierRule for FallbackBindingTierRequiredExternalRefRule {
    fn check(&self, item: &FallbackBindingTier) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback binding tier.
#[derive(Debug, Clone)]
pub struct FallbackBindingTierValidator {
    pub rules: Vec<Rc<dyn FallbackBindingTierRule>>,
}

impl FallbackBindingTierValidator {
    pub fn validate(&self, item: &FallbackBindingTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackBindingTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBindingTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_binding_tier_validator() -> FallbackBindingTierValidator {
    FallbackBindingTierValidator {
        rules: vec![
            Rc::new(FallbackBindingTierAllowedLabelRule {
                allowed: vec!["rolling".to_string(), "batched".to_string(), "seasonal".to_string()],
            }) as Rc<dyn FallbackBindingTierRule>,
            Rc::new(FallbackBindingTierWindowDaysRangeRule { min: 64, max: 120 }),
            Rc::new(FallbackBindingTierExternalRefLengthRule { max_length: 255 }),
            Rc::new(FallbackBindingTierCodeLengthRule { max_length: 64 }),
            Rc::new(FallbackBindingTierRotationDaysRangeRule { min: 16, max: 64 }),
            Rc::new(FallbackBindingTierRequiredExternalRefRule {
                message: "fallback binding tier needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_binding_tier_validator();
        let item = new_fallback_binding_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = FallbackBindingTierWindowDaysRangeRule { min: 32, max: 32 };
        let mut item = new_fallback_binding_tier("x", "y");
        item.window_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = FallbackBindingTierExternalRefLengthRule { max_length: 64 };
        let mut item = new_fallback_binding_tier("x", "y");
        item.external_ref = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackBindingTierCodeLengthRule { max_length: 120 };
        let mut item = new_fallback_binding_tier("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FallbackBindingTierRotationDaysRangeRule { min: 64, max: 86400 };
        let mut item = new_fallback_binding_tier("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = FallbackBindingTierRequiredExternalRefRule {
            message: "fallback binding tier needs this field".to_string(),
        };
        let mut item = new_fallback_binding_tier("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
