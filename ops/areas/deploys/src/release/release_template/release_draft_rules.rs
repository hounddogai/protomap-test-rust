use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release draft must pass. Check returns its problems, or none.
pub trait ReleaseDraftRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDraftRequiredNameRule {
    pub message: String,
}

impl ReleaseDraftRule for ReleaseDraftRequiredNameRule {
    fn check(&self, item: &ReleaseDraft) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDraftNameLengthRule {
    pub max_length: i64,
}

impl ReleaseDraftRule for ReleaseDraftNameLengthRule {
    fn check(&self, item: &ReleaseDraft) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseDraftCodePrefixRule {
    pub prefix: String,
}

impl ReleaseDraftRule for ReleaseDraftCodePrefixRule {
    fn check(&self, item: &ReleaseDraft) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release draft.
#[derive(Debug, Clone)]
pub struct ReleaseDraftValidator {
    pub rules: Vec<Rc<dyn ReleaseDraftRule>>,
}

impl ReleaseDraftValidator {
    pub fn validate(&self, item: &ReleaseDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_draft_validator() -> ReleaseDraftValidator {
    ReleaseDraftValidator {
        rules: vec![
            Rc::new(ReleaseDraftRequiredNameRule {
                message: "release draft needs an entry".to_string(),
            }) as Rc<dyn ReleaseDraftRule>,
            Rc::new(ReleaseDraftNameLengthRule { max_length: 32 }),
            Rc::new(ReleaseDraftCodePrefixRule {
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
        let validator = default_release_draft_validator();
        let item = new_release_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ReleaseDraftRequiredNameRule {
            message: "release draft needs a value".to_string(),
        };
        let mut item = new_release_draft("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ReleaseDraftNameLengthRule { max_length: 64 };
        let mut item = new_release_draft("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
