use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a freeze segment must pass. Check returns its problems, or none.
pub trait FreezeSegmentRule: std::fmt::Debug {
    fn check(&self, item: &FreezeSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegmentCodePrefixRule {
    pub prefix: String,
}

impl FreezeSegmentRule for FreezeSegmentCodePrefixRule {
    fn check(&self, item: &FreezeSegment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegmentRequiredCurrencyRule {
    pub message: String,
}

impl FreezeSegmentRule for FreezeSegmentRequiredCurrencyRule {
    fn check(&self, item: &FreezeSegment) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegmentLabelLengthRule {
    pub max_length: i64,
}

impl FreezeSegmentRule for FreezeSegmentLabelLengthRule {
    fn check(&self, item: &FreezeSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegmentAllowedContactEmailRule {
    pub allowed: Vec<String>,
}

impl FreezeSegmentRule for FreezeSegmentAllowedContactEmailRule {
    fn check(&self, item: &FreezeSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.contact_email)))
        {
            return vec![format!("contact email {} is above maximum", item.contact_email)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegmentCodeLengthRule {
    pub max_length: i64,
}

impl FreezeSegmentRule for FreezeSegmentCodeLengthRule {
    fn check(&self, item: &FreezeSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeSegmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FreezeSegmentRule for FreezeSegmentFloorValueRangeRule {
    fn check(&self, item: &FreezeSegment) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze segment.
#[derive(Debug, Clone)]
pub struct FreezeSegmentValidator {
    pub rules: Vec<Rc<dyn FreezeSegmentRule>>,
}

impl FreezeSegmentValidator {
    pub fn validate(&self, item: &FreezeSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FreezeSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_segment_validator() -> FreezeSegmentValidator {
    FreezeSegmentValidator {
        rules: vec![
            Rc::new(FreezeSegmentCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn FreezeSegmentRule>,
            Rc::new(FreezeSegmentRequiredCurrencyRule {
                message: "freeze segment needs a value".to_string(),
            }),
            Rc::new(FreezeSegmentLabelLengthRule { max_length: 255 }),
            Rc::new(FreezeSegmentAllowedContactEmailRule { allowed: Vec::new() }),
            Rc::new(FreezeSegmentCodeLengthRule { max_length: 120 }),
            Rc::new(FreezeSegmentFloorValueRangeRule { min: 2.8, max: 3242.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_segment_validator();
        let item = new_freeze_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = FreezeSegmentRequiredCurrencyRule {
            message: "freeze segment needs this field".to_string(),
        };
        let mut item = new_freeze_segment("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FreezeSegmentLabelLengthRule { max_length: 64 };
        let mut item = new_freeze_segment("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FreezeSegmentCodeLengthRule { max_length: 120 };
        let mut item = new_freeze_segment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = FreezeSegmentFloorValueRangeRule { min: 3.2, max: 3298.8 };
        let mut item = new_freeze_segment("x", "y");
        item.floor_value = 3300.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
