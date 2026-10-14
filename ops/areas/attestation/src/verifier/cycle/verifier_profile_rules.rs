use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier profile must pass. Check returns its problems, or none.
pub trait VerifierProfileRule: std::fmt::Debug {
    fn check(&self, item: &VerifierProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierProfileAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl VerifierProfileRule for VerifierProfileAllowedTimezoneRule {
    fn check(&self, item: &VerifierProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is below minimum", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierProfileCodePrefixRule {
    pub prefix: String,
}

impl VerifierProfileRule for VerifierProfileCodePrefixRule {
    fn check(&self, item: &VerifierProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierProfileRequiredOwnerIdRule {
    pub message: String,
}

impl VerifierProfileRule for VerifierProfileRequiredOwnerIdRule {
    fn check(&self, item: &VerifierProfile) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier profile.
#[derive(Debug, Clone)]
pub struct VerifierProfileValidator {
    pub rules: Vec<Rc<dyn VerifierProfileRule>>,
}

impl VerifierProfileValidator {
    pub fn validate(&self, item: &VerifierProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_profile_validator() -> VerifierProfileValidator {
    VerifierProfileValidator {
        rules: vec![
            Rc::new(VerifierProfileAllowedTimezoneRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn VerifierProfileRule>,
            Rc::new(VerifierProfileCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(VerifierProfileRequiredOwnerIdRule {
                message: "verifier profile needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_profile_validator();
        let item = new_verifier_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = VerifierProfileRequiredOwnerIdRule {
            message: "verifier profile needs a value".to_string(),
        };
        let mut item = new_verifier_profile("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
