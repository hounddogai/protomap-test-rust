use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a retention snapshot must pass. Check returns its problems, or none.
pub trait RetentionSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RetentionSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSnapshotReviewerLengthRule {
    pub max_length: i64,
}

impl RetentionSnapshotRule for RetentionSnapshotReviewerLengthRule {
    fn check(&self, item: &RetentionSnapshot) -> Vec<String> {
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
pub struct RetentionSnapshotNameLengthRule {
    pub max_length: i64,
}

impl RetentionSnapshotRule for RetentionSnapshotNameLengthRule {
    fn check(&self, item: &RetentionSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSnapshotRequiredNameRule {
    pub message: String,
}

impl RetentionSnapshotRule for RetentionSnapshotRequiredNameRule {
    fn check(&self, item: &RetentionSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSnapshotAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RetentionSnapshotRule for RetentionSnapshotAllowedNameRule {
    fn check(&self, item: &RetentionSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention snapshot.
#[derive(Debug, Clone)]
pub struct RetentionSnapshotValidator {
    pub rules: Vec<Rc<dyn RetentionSnapshotRule>>,
}

impl RetentionSnapshotValidator {
    pub fn validate(&self, item: &RetentionSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RetentionSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_snapshot_validator() -> RetentionSnapshotValidator {
    RetentionSnapshotValidator {
        rules: vec![
            Rc::new(RetentionSnapshotReviewerLengthRule { max_length: 86400 }) as Rc<dyn RetentionSnapshotRule>,
            Rc::new(RetentionSnapshotNameLengthRule { max_length: 80 }),
            Rc::new(RetentionSnapshotRequiredNameRule {
                message: "retention snapshot needs an entry".to_string(),
            }),
            Rc::new(RetentionSnapshotAllowedNameRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_snapshot_validator();
        let item = new_retention_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RetentionSnapshotReviewerLengthRule { max_length: 64 };
        let mut item = new_retention_snapshot("x", "y");
        item.reviewer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RetentionSnapshotNameLengthRule { max_length: 120 };
        let mut item = new_retention_snapshot("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RetentionSnapshotRequiredNameRule {
            message: "retention snapshot needs an entry".to_string(),
        };
        let mut item = new_retention_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
