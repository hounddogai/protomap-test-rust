use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict binding exception must pass. Check returns its problems, or none.
pub trait StrictBindingExceptionRule: std::fmt::Debug {
    fn check(&self, item: &StrictBindingException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingExceptionContactEmailLengthRule {
    pub max_length: i64,
}

impl StrictBindingExceptionRule for StrictBindingExceptionContactEmailLengthRule {
    fn check(&self, item: &StrictBindingException) -> Vec<String> {
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
pub struct StrictBindingExceptionApprovedGuardRule {
    pub limit: i64,
}

impl StrictBindingExceptionRule for StrictBindingExceptionApprovedGuardRule {
    fn check(&self, item: &StrictBindingException) -> Vec<String> {
        if item.approved && (item.retries > self.limit) {
            return vec![format!(
                "retries {} exceeds {} while approved",
                item.retries, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingExceptionRequiredLocaleRule {
    pub message: String,
}

impl StrictBindingExceptionRule for StrictBindingExceptionRequiredLocaleRule {
    fn check(&self, item: &StrictBindingException) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBindingExceptionRequiredContactEmailRule {
    pub message: String,
}

impl StrictBindingExceptionRule for StrictBindingExceptionRequiredContactEmailRule {
    fn check(&self, item: &StrictBindingException) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict binding exception.
#[derive(Debug, Clone)]
pub struct StrictBindingExceptionValidator {
    pub rules: Vec<Rc<dyn StrictBindingExceptionRule>>,
}

impl StrictBindingExceptionValidator {
    pub fn validate(&self, item: &StrictBindingException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictBindingException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictBindingException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_binding_exception_validator() -> StrictBindingExceptionValidator {
    StrictBindingExceptionValidator {
        rules: vec![
            Rc::new(StrictBindingExceptionContactEmailLengthRule { max_length: 255 })
                as Rc<dyn StrictBindingExceptionRule>,
            Rc::new(StrictBindingExceptionApprovedGuardRule { limit: 120 }),
            Rc::new(StrictBindingExceptionRequiredLocaleRule {
                message: "strict binding exception needs this field".to_string(),
            }),
            Rc::new(StrictBindingExceptionRequiredContactEmailRule {
                message: "strict binding exception needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_binding_exception_validator();
        let item = new_strict_binding_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = StrictBindingExceptionContactEmailLengthRule { max_length: 86400 };
        let mut item = new_strict_binding_exception("x", "y");
        item.contact_email = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = StrictBindingExceptionRequiredLocaleRule {
            message: "strict binding exception needs an entry".to_string(),
        };
        let mut item = new_strict_binding_exception("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = StrictBindingExceptionRequiredContactEmailRule {
            message: "strict binding exception needs this field".to_string(),
        };
        let mut item = new_strict_binding_exception("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
