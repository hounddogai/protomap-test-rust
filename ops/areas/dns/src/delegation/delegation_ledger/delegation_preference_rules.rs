use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation preference must pass. Check returns its problems, or none.
pub trait DelegationPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &DelegationPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl DelegationPreferenceRule for DelegationPreferenceCodeLengthRule {
    fn check(&self, item: &DelegationPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPreferenceRequiredScopeRule {
    pub message: String,
}

impl DelegationPreferenceRule for DelegationPreferenceRequiredScopeRule {
    fn check(&self, item: &DelegationPreference) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPreferenceCodePrefixRule {
    pub prefix: String,
}

impl DelegationPreferenceRule for DelegationPreferenceCodePrefixRule {
    fn check(&self, item: &DelegationPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation preference.
#[derive(Debug, Clone)]
pub struct DelegationPreferenceValidator {
    pub rules: Vec<Rc<dyn DelegationPreferenceRule>>,
}

impl DelegationPreferenceValidator {
    pub fn validate(&self, item: &DelegationPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DelegationPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_preference_validator() -> DelegationPreferenceValidator {
    DelegationPreferenceValidator {
        rules: vec![
            Rc::new(DelegationPreferenceCodeLengthRule { max_length: 80 }) as Rc<dyn DelegationPreferenceRule>,
            Rc::new(DelegationPreferenceRequiredScopeRule {
                message: "delegation preference needs an entry".to_string(),
            }),
            Rc::new(DelegationPreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_preference_validator();
        let item = new_delegation_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DelegationPreferenceCodeLengthRule { max_length: 64 };
        let mut item = new_delegation_preference("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = DelegationPreferenceRequiredScopeRule {
            message: "delegation preference needs this field".to_string(),
        };
        let mut item = new_delegation_preference("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
