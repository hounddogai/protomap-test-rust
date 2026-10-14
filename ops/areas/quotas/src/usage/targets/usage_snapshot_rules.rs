use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a usage snapshot must pass. Check returns its problems, or none.
pub trait UsageSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &UsageSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshotRequiredSourceRule {
    pub message: String,
}

impl UsageSnapshotRule for UsageSnapshotRequiredSourceRule {
    fn check(&self, item: &UsageSnapshot) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshotRequiredTitleRule {
    pub message: String,
}

impl UsageSnapshotRule for UsageSnapshotRequiredTitleRule {
    fn check(&self, item: &UsageSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshotReviewerLengthRule {
    pub max_length: i64,
}

impl UsageSnapshotRule for UsageSnapshotReviewerLengthRule {
    fn check(&self, item: &UsageSnapshot) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl UsageSnapshotRule for UsageSnapshotTitleLengthRule {
    fn check(&self, item: &UsageSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageSnapshotSourceLengthRule {
    pub max_length: i64,
}

impl UsageSnapshotRule for UsageSnapshotSourceLengthRule {
    fn check(&self, item: &UsageSnapshot) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage snapshot.
#[derive(Debug, Clone)]
pub struct UsageSnapshotValidator {
    pub rules: Vec<Rc<dyn UsageSnapshotRule>>,
}

impl UsageSnapshotValidator {
    pub fn validate(&self, item: &UsageSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &UsageSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_snapshot_validator() -> UsageSnapshotValidator {
    UsageSnapshotValidator {
        rules: vec![
            Rc::new(UsageSnapshotRequiredSourceRule {
                message: "usage snapshot needs a value".to_string(),
            }) as Rc<dyn UsageSnapshotRule>,
            Rc::new(UsageSnapshotRequiredTitleRule {
                message: "usage snapshot needs this field".to_string(),
            }),
            Rc::new(UsageSnapshotReviewerLengthRule { max_length: 64 }),
            Rc::new(UsageSnapshotTitleLengthRule { max_length: 80 }),
            Rc::new(UsageSnapshotSourceLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_snapshot_validator();
        let item = new_usage_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = UsageSnapshotRequiredSourceRule {
            message: "usage snapshot needs an entry".to_string(),
        };
        let mut item = new_usage_snapshot("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = UsageSnapshotRequiredTitleRule {
            message: "usage snapshot needs a value".to_string(),
        };
        let mut item = new_usage_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = UsageSnapshotReviewerLengthRule { max_length: 80 };
        let mut item = new_usage_snapshot("x", "y");
        item.reviewer = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = UsageSnapshotTitleLengthRule { max_length: 32 };
        let mut item = new_usage_snapshot("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = UsageSnapshotSourceLengthRule { max_length: 80 };
        let mut item = new_usage_snapshot("x", "y");
        item.source = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
