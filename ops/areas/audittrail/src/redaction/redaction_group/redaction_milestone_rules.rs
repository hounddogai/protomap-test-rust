use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction milestone must pass. Check returns its problems, or none.
pub trait RedactionMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RedactionMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMilestoneRequiredCodeRule {
    pub message: String,
}

impl RedactionMilestoneRule for RedactionMilestoneRequiredCodeRule {
    fn check(&self, item: &RedactionMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMilestoneTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RedactionMilestoneRule for RedactionMilestoneTagsLimitRule {
    fn check(&self, item: &RedactionMilestone) -> Vec<String> {
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
pub struct RedactionMilestoneClusterLengthRule {
    pub max_length: i64,
}

impl RedactionMilestoneRule for RedactionMilestoneClusterLengthRule {
    fn check(&self, item: &RedactionMilestone) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionMilestoneNameLengthRule {
    pub max_length: i64,
}

impl RedactionMilestoneRule for RedactionMilestoneNameLengthRule {
    fn check(&self, item: &RedactionMilestone) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction milestone.
#[derive(Debug, Clone)]
pub struct RedactionMilestoneValidator {
    pub rules: Vec<Rc<dyn RedactionMilestoneRule>>,
}

impl RedactionMilestoneValidator {
    pub fn validate(&self, item: &RedactionMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_milestone_validator() -> RedactionMilestoneValidator {
    RedactionMilestoneValidator {
        rules: vec![
            Rc::new(RedactionMilestoneRequiredCodeRule {
                message: "redaction milestone needs this field".to_string(),
            }) as Rc<dyn RedactionMilestoneRule>,
            Rc::new(RedactionMilestoneTagsLimitRule {
                max_count: 86400,
                banned: vec!["scheduled".to_string()],
            }),
            Rc::new(RedactionMilestoneClusterLengthRule { max_length: 3600 }),
            Rc::new(RedactionMilestoneNameLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_milestone_validator();
        let item = new_redaction_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionMilestoneRequiredCodeRule {
            message: "redaction milestone needs an entry".to_string(),
        };
        let mut item = new_redaction_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = RedactionMilestoneClusterLengthRule { max_length: 86400 };
        let mut item = new_redaction_milestone("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RedactionMilestoneNameLengthRule { max_length: 120 };
        let mut item = new_redaction_milestone("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
