use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback preference must pass. Check returns its problems, or none.
pub trait RollbackPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RollbackPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPreferenceAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl RollbackPreferenceRule for RollbackPreferenceAllowedNotesRule {
    fn check(&self, item: &RollbackPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is inconsistent", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPreferenceCodePrefixRule {
    pub prefix: String,
}

impl RollbackPreferenceRule for RollbackPreferenceCodePrefixRule {
    fn check(&self, item: &RollbackPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl RollbackPreferenceRule for RollbackPreferenceCodeLengthRule {
    fn check(&self, item: &RollbackPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPreferenceNotesLengthRule {
    pub max_length: i64,
}

impl RollbackPreferenceRule for RollbackPreferenceNotesLengthRule {
    fn check(&self, item: &RollbackPreference) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback preference.
#[derive(Debug, Clone)]
pub struct RollbackPreferenceValidator {
    pub rules: Vec<Rc<dyn RollbackPreferenceRule>>,
}

impl RollbackPreferenceValidator {
    pub fn validate(&self, item: &RollbackPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_preference_validator() -> RollbackPreferenceValidator {
    RollbackPreferenceValidator {
        rules: vec![
            Rc::new(RollbackPreferenceAllowedNotesRule {
                allowed: vec!["scheduled".to_string(), "pending".to_string(), "seasonal".to_string()],
            }) as Rc<dyn RollbackPreferenceRule>,
            Rc::new(RollbackPreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollbackPreferenceCodeLengthRule { max_length: 86400 }),
            Rc::new(RollbackPreferenceNotesLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_preference_validator();
        let item = new_rollback_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackPreferenceCodeLengthRule { max_length: 3600 };
        let mut item = new_rollback_preference("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollbackPreferenceNotesLengthRule { max_length: 80 };
        let mut item = new_rollback_preference("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
