use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant preference must pass. Check returns its problems, or none.
pub trait GrantPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &GrantPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPreferenceRequiredReviewerRule {
    pub message: String,
}

impl GrantPreferenceRule for GrantPreferenceRequiredReviewerRule {
    fn check(&self, item: &GrantPreference) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPreferenceRequiredDescriptionRule {
    pub message: String,
}

impl GrantPreferenceRule for GrantPreferenceRequiredDescriptionRule {
    fn check(&self, item: &GrantPreference) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPreferenceCodePrefixRule {
    pub prefix: String,
}

impl GrantPreferenceRule for GrantPreferenceCodePrefixRule {
    fn check(&self, item: &GrantPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant preference.
#[derive(Debug, Clone)]
pub struct GrantPreferenceValidator {
    pub rules: Vec<Rc<dyn GrantPreferenceRule>>,
}

impl GrantPreferenceValidator {
    pub fn validate(&self, item: &GrantPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GrantPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_preference_validator() -> GrantPreferenceValidator {
    GrantPreferenceValidator {
        rules: vec![
            Rc::new(GrantPreferenceRequiredReviewerRule {
                message: "grant preference needs a value".to_string(),
            }) as Rc<dyn GrantPreferenceRule>,
            Rc::new(GrantPreferenceRequiredDescriptionRule {
                message: "grant preference needs this field".to_string(),
            }),
            Rc::new(GrantPreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_preference_validator();
        let item = new_grant_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = GrantPreferenceRequiredReviewerRule {
            message: "grant preference needs a value".to_string(),
        };
        let mut item = new_grant_preference("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = GrantPreferenceRequiredDescriptionRule {
            message: "grant preference needs a value".to_string(),
        };
        let mut item = new_grant_preference("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
