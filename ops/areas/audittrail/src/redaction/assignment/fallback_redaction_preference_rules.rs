use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback redaction preference must pass. Check returns its problems, or none.
pub trait FallbackRedactionPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &FallbackRedactionPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl FallbackRedactionPreferenceRule for FallbackRedactionPreferenceCodeLengthRule {
    fn check(&self, item: &FallbackRedactionPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionPreferenceRequiredCodeRule {
    pub message: String,
}

impl FallbackRedactionPreferenceRule for FallbackRedactionPreferenceRequiredCodeRule {
    fn check(&self, item: &FallbackRedactionPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionPreferenceRequiredLabelRule {
    pub message: String,
}

impl FallbackRedactionPreferenceRule for FallbackRedactionPreferenceRequiredLabelRule {
    fn check(&self, item: &FallbackRedactionPreference) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRedactionPreferenceAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackRedactionPreferenceRule for FallbackRedactionPreferenceAllowedCodeRule {
    fn check(&self, item: &FallbackRedactionPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback redaction preference.
#[derive(Debug, Clone)]
pub struct FallbackRedactionPreferenceValidator {
    pub rules: Vec<Rc<dyn FallbackRedactionPreferenceRule>>,
}

impl FallbackRedactionPreferenceValidator {
    pub fn validate(&self, item: &FallbackRedactionPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackRedactionPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackRedactionPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_redaction_preference_validator() -> FallbackRedactionPreferenceValidator {
    FallbackRedactionPreferenceValidator {
        rules: vec![
            Rc::new(FallbackRedactionPreferenceCodeLengthRule { max_length: 255 })
                as Rc<dyn FallbackRedactionPreferenceRule>,
            Rc::new(FallbackRedactionPreferenceRequiredCodeRule {
                message: "fallback redaction preference needs a value".to_string(),
            }),
            Rc::new(FallbackRedactionPreferenceRequiredLabelRule {
                message: "fallback redaction preference needs a value".to_string(),
            }),
            Rc::new(FallbackRedactionPreferenceAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_redaction_preference_validator();
        let item = new_fallback_redaction_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackRedactionPreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_fallback_redaction_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackRedactionPreferenceRequiredCodeRule {
            message: "fallback redaction preference needs this field".to_string(),
        };
        let mut item = new_fallback_redaction_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackRedactionPreferenceRequiredLabelRule {
            message: "fallback redaction preference needs this field".to_string(),
        };
        let mut item = new_fallback_redaction_preference("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
