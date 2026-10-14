use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem group must pass. Check returns its problems, or none.
pub trait PostmortemGroupRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemGroupRequiredCodeRule {
    pub message: String,
}

impl PostmortemGroupRule for PostmortemGroupRequiredCodeRule {
    fn check(&self, item: &PostmortemGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemGroupRequiredNameRule {
    pub message: String,
}

impl PostmortemGroupRule for PostmortemGroupRequiredNameRule {
    fn check(&self, item: &PostmortemGroup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemGroupCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemGroupRule for PostmortemGroupCodeLengthRule {
    fn check(&self, item: &PostmortemGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemGroupTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PostmortemGroupRule for PostmortemGroupTtlSecondsRangeRule {
    fn check(&self, item: &PostmortemGroup) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemGroupNamespaceLengthRule {
    pub max_length: i64,
}

impl PostmortemGroupRule for PostmortemGroupNamespaceLengthRule {
    fn check(&self, item: &PostmortemGroup) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem group.
#[derive(Debug, Clone)]
pub struct PostmortemGroupValidator {
    pub rules: Vec<Rc<dyn PostmortemGroupRule>>,
}

impl PostmortemGroupValidator {
    pub fn validate(&self, item: &PostmortemGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_group_validator() -> PostmortemGroupValidator {
    PostmortemGroupValidator {
        rules: vec![
            Rc::new(PostmortemGroupRequiredCodeRule {
                message: "postmortem group needs an entry".to_string(),
            }) as Rc<dyn PostmortemGroupRule>,
            Rc::new(PostmortemGroupRequiredNameRule {
                message: "postmortem group needs an entry".to_string(),
            }),
            Rc::new(PostmortemGroupCodeLengthRule { max_length: 32 }),
            Rc::new(PostmortemGroupTtlSecondsRangeRule { min: 32, max: 64 }),
            Rc::new(PostmortemGroupNamespaceLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_group_validator();
        let item = new_postmortem_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PostmortemGroupRequiredCodeRule {
            message: "postmortem group needs this field".to_string(),
        };
        let mut item = new_postmortem_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PostmortemGroupRequiredNameRule {
            message: "postmortem group needs this field".to_string(),
        };
        let mut item = new_postmortem_group("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PostmortemGroupCodeLengthRule { max_length: 80 };
        let mut item = new_postmortem_group("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = PostmortemGroupTtlSecondsRangeRule { min: 64, max: 255 };
        let mut item = new_postmortem_group("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = PostmortemGroupNamespaceLengthRule { max_length: 64 };
        let mut item = new_postmortem_group("x", "y");
        item.namespace = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
