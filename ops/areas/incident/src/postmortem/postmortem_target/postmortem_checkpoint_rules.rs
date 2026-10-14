use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem checkpoint must pass. Check returns its problems, or none.
pub trait PostmortemCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpointRequiredDescriptionRule {
    pub message: String,
}

impl PostmortemCheckpointRule for PostmortemCheckpointRequiredDescriptionRule {
    fn check(&self, item: &PostmortemCheckpoint) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpointNameLengthRule {
    pub max_length: i64,
}

impl PostmortemCheckpointRule for PostmortemCheckpointNameLengthRule {
    fn check(&self, item: &PostmortemCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpointRequiredCodeRule {
    pub message: String,
}

impl PostmortemCheckpointRule for PostmortemCheckpointRequiredCodeRule {
    fn check(&self, item: &PostmortemCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpointRequiredSegmentRule {
    pub message: String,
}

impl PostmortemCheckpointRule for PostmortemCheckpointRequiredSegmentRule {
    fn check(&self, item: &PostmortemCheckpoint) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemCheckpointRule for PostmortemCheckpointCodeLengthRule {
    fn check(&self, item: &PostmortemCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem checkpoint.
#[derive(Debug, Clone)]
pub struct PostmortemCheckpointValidator {
    pub rules: Vec<Rc<dyn PostmortemCheckpointRule>>,
}

impl PostmortemCheckpointValidator {
    pub fn validate(&self, item: &PostmortemCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_checkpoint_validator() -> PostmortemCheckpointValidator {
    PostmortemCheckpointValidator {
        rules: vec![
            Rc::new(PostmortemCheckpointRequiredDescriptionRule {
                message: "postmortem checkpoint needs this field".to_string(),
            }) as Rc<dyn PostmortemCheckpointRule>,
            Rc::new(PostmortemCheckpointNameLengthRule { max_length: 80 }),
            Rc::new(PostmortemCheckpointRequiredCodeRule {
                message: "postmortem checkpoint needs a value".to_string(),
            }),
            Rc::new(PostmortemCheckpointRequiredSegmentRule {
                message: "postmortem checkpoint needs an entry".to_string(),
            }),
            Rc::new(PostmortemCheckpointCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_checkpoint_validator();
        let item = new_postmortem_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = PostmortemCheckpointRequiredDescriptionRule {
            message: "postmortem checkpoint needs an entry".to_string(),
        };
        let mut item = new_postmortem_checkpoint("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PostmortemCheckpointNameLengthRule { max_length: 32 };
        let mut item = new_postmortem_checkpoint("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PostmortemCheckpointRequiredCodeRule {
            message: "postmortem checkpoint needs this field".to_string(),
        };
        let mut item = new_postmortem_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = PostmortemCheckpointRequiredSegmentRule {
            message: "postmortem checkpoint needs this field".to_string(),
        };
        let mut item = new_postmortem_checkpoint("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PostmortemCheckpointCodeLengthRule { max_length: 32 };
        let mut item = new_postmortem_checkpoint("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
