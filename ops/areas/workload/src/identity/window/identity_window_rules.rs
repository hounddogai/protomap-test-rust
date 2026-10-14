use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a identity window must pass. Check returns its problems, or none.
pub trait IdentityWindowRule: std::fmt::Debug {
    fn check(&self, item: &IdentityWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityWindowAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl IdentityWindowRule for IdentityWindowAllowedCountryRule {
    fn check(&self, item: &IdentityWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is not allowed", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityWindowRequiredLabelRule {
    pub message: String,
}

impl IdentityWindowRule for IdentityWindowRequiredLabelRule {
    fn check(&self, item: &IdentityWindow) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityWindowCountryLengthRule {
    pub max_length: i64,
}

impl IdentityWindowRule for IdentityWindowCountryLengthRule {
    fn check(&self, item: &IdentityWindow) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a identity window.
#[derive(Debug, Clone)]
pub struct IdentityWindowValidator {
    pub rules: Vec<Rc<dyn IdentityWindowRule>>,
}

impl IdentityWindowValidator {
    pub fn validate(&self, item: &IdentityWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_window_validator() -> IdentityWindowValidator {
    IdentityWindowValidator {
        rules: vec![
            Rc::new(IdentityWindowAllowedCountryRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn IdentityWindowRule>,
            Rc::new(IdentityWindowRequiredLabelRule {
                message: "identity window needs this field".to_string(),
            }),
            Rc::new(IdentityWindowCountryLengthRule { max_length: 86400 }),
        ],
    }
}
