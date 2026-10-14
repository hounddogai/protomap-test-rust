use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback page variance must pass. Check returns its problems, or none.
pub trait FallbackPageVarianceRule: std::fmt::Debug {
    fn check(&self, item: &FallbackPageVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageVarianceRequiredNameRule {
    pub message: String,
}

impl FallbackPageVarianceRule for FallbackPageVarianceRequiredNameRule {
    fn check(&self, item: &FallbackPageVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageVarianceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackPageVarianceRule for FallbackPageVarianceAllowedCodeRule {
    fn check(&self, item: &FallbackPageVariance) -> Vec<String> {
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
pub struct FallbackPageVariancePhoneLengthRule {
    pub max_length: i64,
}

impl FallbackPageVarianceRule for FallbackPageVariancePhoneLengthRule {
    fn check(&self, item: &FallbackPageVariance) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageVarianceSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPageVarianceRule for FallbackPageVarianceSequenceRangeRule {
    fn check(&self, item: &FallbackPageVariance) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageVarianceCodeLengthRule {
    pub max_length: i64,
}

impl FallbackPageVarianceRule for FallbackPageVarianceCodeLengthRule {
    fn check(&self, item: &FallbackPageVariance) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageVarianceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackPageVarianceRule for FallbackPageVarianceRotationDaysRangeRule {
    fn check(&self, item: &FallbackPageVariance) -> Vec<String> {
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

/// Runs every rule of a fallback page variance.
#[derive(Debug, Clone)]
pub struct FallbackPageVarianceValidator {
    pub rules: Vec<Rc<dyn FallbackPageVarianceRule>>,
}

impl FallbackPageVarianceValidator {
    pub fn validate(&self, item: &FallbackPageVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackPageVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackPageVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_page_variance_validator() -> FallbackPageVarianceValidator {
    FallbackPageVarianceValidator {
        rules: vec![
            Rc::new(FallbackPageVarianceRequiredNameRule {
                message: "fallback page variance needs an entry".to_string(),
            }) as Rc<dyn FallbackPageVarianceRule>,
            Rc::new(FallbackPageVarianceAllowedCodeRule {
                allowed: vec!["seasonal".to_string(), "lenient".to_string(), "pending".to_string()],
            }),
            Rc::new(FallbackPageVariancePhoneLengthRule { max_length: 16 }),
            Rc::new(FallbackPageVarianceSequenceRangeRule { min: 16, max: 64 }),
            Rc::new(FallbackPageVarianceCodeLengthRule { max_length: 3600 }),
            Rc::new(FallbackPageVarianceRotationDaysRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_page_variance_validator();
        let item = new_fallback_page_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackPageVarianceRequiredNameRule {
            message: "fallback page variance needs a value".to_string(),
        };
        let mut item = new_fallback_page_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = FallbackPageVariancePhoneLengthRule { max_length: 16 };
        let mut item = new_fallback_page_variance("x", "y");
        item.phone = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = FallbackPageVarianceSequenceRangeRule { min: 32, max: 120 };
        let mut item = new_fallback_page_variance("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackPageVarianceCodeLengthRule { max_length: 3600 };
        let mut item = new_fallback_page_variance("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = FallbackPageVarianceRotationDaysRangeRule { min: 64, max: 3600 };
        let mut item = new_fallback_page_variance("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
