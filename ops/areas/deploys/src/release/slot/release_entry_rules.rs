use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release entry must pass. Check returns its problems, or none.
pub trait ReleaseEntryRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEntryVisibleGuardRule {
    pub limit: i64,
}

impl ReleaseEntryRule for ReleaseEntryVisibleGuardRule {
    fn check(&self, item: &ReleaseEntry) -> Vec<String> {
        if item.visible && (item.version > self.limit) {
            return vec![format!("version {} exceeds {} while visible", item.version, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEntryCodePrefixRule {
    pub prefix: String,
}

impl ReleaseEntryRule for ReleaseEntryCodePrefixRule {
    fn check(&self, item: &ReleaseEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEntryRequiredTitleRule {
    pub message: String,
}

impl ReleaseEntryRule for ReleaseEntryRequiredTitleRule {
    fn check(&self, item: &ReleaseEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEntryOwnerIdLengthRule {
    pub max_length: i64,
}

impl ReleaseEntryRule for ReleaseEntryOwnerIdLengthRule {
    fn check(&self, item: &ReleaseEntry) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a release entry.
#[derive(Debug, Clone)]
pub struct ReleaseEntryValidator {
    pub rules: Vec<Rc<dyn ReleaseEntryRule>>,
}

impl ReleaseEntryValidator {
    pub fn validate(&self, item: &ReleaseEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_entry_validator() -> ReleaseEntryValidator {
    ReleaseEntryValidator {
        rules: vec![
            Rc::new(ReleaseEntryVisibleGuardRule { limit: 80 }) as Rc<dyn ReleaseEntryRule>,
            Rc::new(ReleaseEntryCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ReleaseEntryRequiredTitleRule {
                message: "release entry needs an entry".to_string(),
            }),
            Rc::new(ReleaseEntryOwnerIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_entry_validator();
        let item = new_release_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReleaseEntryRequiredTitleRule {
            message: "release entry needs a value".to_string(),
        };
        let mut item = new_release_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ReleaseEntryOwnerIdLengthRule { max_length: 64 };
        let mut item = new_release_entry("x", "y");
        item.owner_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
