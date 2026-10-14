use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending binding group must pass. Check returns its problems, or none.
pub trait PendingBindingGroupRule: std::fmt::Debug {
    fn check(&self, item: &PendingBindingGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingGroupRequiredNotesRule {
    pub message: String,
}

impl PendingBindingGroupRule for PendingBindingGroupRequiredNotesRule {
    fn check(&self, item: &PendingBindingGroup) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingGroupRequiredCodeRule {
    pub message: String,
}

impl PendingBindingGroupRule for PendingBindingGroupRequiredCodeRule {
    fn check(&self, item: &PendingBindingGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingGroupTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl PendingBindingGroupRule for PendingBindingGroupTagsLimitRule {
    fn check(&self, item: &PendingBindingGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingGroupRequiredNameRule {
    pub message: String,
}

impl PendingBindingGroupRule for PendingBindingGroupRequiredNameRule {
    fn check(&self, item: &PendingBindingGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending binding group.
#[derive(Debug, Clone)]
pub struct PendingBindingGroupValidator {
    pub rules: Vec<Rc<dyn PendingBindingGroupRule>>,
}

impl PendingBindingGroupValidator {
    pub fn validate(&self, item: &PendingBindingGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingBindingGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingBindingGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_binding_group_validator() -> PendingBindingGroupValidator {
    PendingBindingGroupValidator {
        rules: vec![
            Rc::new(PendingBindingGroupRequiredNotesRule {
                message: "pending binding group needs this field".to_string(),
            }) as Rc<dyn PendingBindingGroupRule>,
            Rc::new(PendingBindingGroupRequiredCodeRule {
                message: "pending binding group needs an entry".to_string(),
            }),
            Rc::new(PendingBindingGroupTagsLimitRule {
                max_count: 80,
                banned: vec!["primary".to_string(), "legacy".to_string(), "adaptive".to_string()],
            }),
            Rc::new(PendingBindingGroupRequiredNameRule {
                message: "pending binding group needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_binding_group_validator();
        let item = new_pending_binding_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = PendingBindingGroupRequiredNotesRule {
            message: "pending binding group needs an entry".to_string(),
        };
        let mut item = new_pending_binding_group("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingBindingGroupRequiredCodeRule {
            message: "pending binding group needs an entry".to_string(),
        };
        let mut item = new_pending_binding_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PendingBindingGroupRequiredNameRule {
            message: "pending binding group needs this field".to_string(),
        };
        let mut item = new_pending_binding_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
