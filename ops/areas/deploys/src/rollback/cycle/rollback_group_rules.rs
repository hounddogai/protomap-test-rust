use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback group must pass. Check returns its problems, or none.
pub trait RollbackGroupRule: std::fmt::Debug {
    fn check(&self, item: &RollbackGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroupCodePrefixRule {
    pub prefix: String,
}

impl RollbackGroupRule for RollbackGroupCodePrefixRule {
    fn check(&self, item: &RollbackGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroupRequiredNameRule {
    pub message: String,
}

impl RollbackGroupRule for RollbackGroupRequiredNameRule {
    fn check(&self, item: &RollbackGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroupCodeLengthRule {
    pub max_length: i64,
}

impl RollbackGroupRule for RollbackGroupCodeLengthRule {
    fn check(&self, item: &RollbackGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroupRequiredCodeRule {
    pub message: String,
}

impl RollbackGroupRule for RollbackGroupRequiredCodeRule {
    fn check(&self, item: &RollbackGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackGroupTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RollbackGroupRule for RollbackGroupTagsLimitRule {
    fn check(&self, item: &RollbackGroup) -> Vec<String> {
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
pub struct RollbackGroupNameLengthRule {
    pub max_length: i64,
}

impl RollbackGroupRule for RollbackGroupNameLengthRule {
    fn check(&self, item: &RollbackGroup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback group.
#[derive(Debug, Clone)]
pub struct RollbackGroupValidator {
    pub rules: Vec<Rc<dyn RollbackGroupRule>>,
}

impl RollbackGroupValidator {
    pub fn validate(&self, item: &RollbackGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_group_validator() -> RollbackGroupValidator {
    RollbackGroupValidator {
        rules: vec![
            Rc::new(RollbackGroupCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn RollbackGroupRule>,
            Rc::new(RollbackGroupRequiredNameRule {
                message: "rollback group needs an entry".to_string(),
            }),
            Rc::new(RollbackGroupCodeLengthRule { max_length: 255 }),
            Rc::new(RollbackGroupRequiredCodeRule {
                message: "rollback group needs an entry".to_string(),
            }),
            Rc::new(RollbackGroupTagsLimitRule {
                max_count: 86400,
                banned: vec!["weighted".to_string(), "global".to_string()],
            }),
            Rc::new(RollbackGroupNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_group_validator();
        let item = new_rollback_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollbackGroupRequiredNameRule {
            message: "rollback group needs an entry".to_string(),
        };
        let mut item = new_rollback_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackGroupCodeLengthRule { max_length: 86400 };
        let mut item = new_rollback_group("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackGroupRequiredCodeRule {
            message: "rollback group needs an entry".to_string(),
        };
        let mut item = new_rollback_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollbackGroupNameLengthRule { max_length: 1000 };
        let mut item = new_rollback_group("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
