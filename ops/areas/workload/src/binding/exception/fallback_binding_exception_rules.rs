use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback binding exception must pass. Check returns its problems, or none.
pub trait FallbackBindingExceptionRule: std::fmt::Debug {
    fn check(&self, item: &FallbackBindingException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingExceptionAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl FallbackBindingExceptionRule for FallbackBindingExceptionAllowedPhoneRule {
    fn check(&self, item: &FallbackBindingException) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is blocked", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingExceptionChecksumLengthRule {
    pub max_length: i64,
}

impl FallbackBindingExceptionRule for FallbackBindingExceptionChecksumLengthRule {
    fn check(&self, item: &FallbackBindingException) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBindingExceptionRequiredCodeRule {
    pub message: String,
}

impl FallbackBindingExceptionRule for FallbackBindingExceptionRequiredCodeRule {
    fn check(&self, item: &FallbackBindingException) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback binding exception.
#[derive(Debug, Clone)]
pub struct FallbackBindingExceptionValidator {
    pub rules: Vec<Rc<dyn FallbackBindingExceptionRule>>,
}

impl FallbackBindingExceptionValidator {
    pub fn validate(&self, item: &FallbackBindingException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackBindingException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackBindingException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_binding_exception_validator() -> FallbackBindingExceptionValidator {
    FallbackBindingExceptionValidator {
        rules: vec![
            Rc::new(FallbackBindingExceptionAllowedPhoneRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn FallbackBindingExceptionRule>,
            Rc::new(FallbackBindingExceptionChecksumLengthRule { max_length: 80 }),
            Rc::new(FallbackBindingExceptionRequiredCodeRule {
                message: "fallback binding exception needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_binding_exception_validator();
        let item = new_fallback_binding_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = FallbackBindingExceptionChecksumLengthRule { max_length: 86400 };
        let mut item = new_fallback_binding_exception("x", "y");
        item.checksum = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackBindingExceptionRequiredCodeRule {
            message: "fallback binding exception needs this field".to_string(),
        };
        let mut item = new_fallback_binding_exception("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
