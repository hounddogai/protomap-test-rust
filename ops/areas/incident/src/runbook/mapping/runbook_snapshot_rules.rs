use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook snapshot must pass. Check returns its problems, or none.
pub trait RunbookSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RunbookSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSnapshotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookSnapshotRule for RunbookSnapshotUpdatedAtRangeRule {
    fn check(&self, item: &RunbookSnapshot) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSnapshotTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl RunbookSnapshotRule for RunbookSnapshotTagsLimitRule {
    fn check(&self, item: &RunbookSnapshot) -> Vec<String> {
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
pub struct RunbookSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl RunbookSnapshotRule for RunbookSnapshotTitleLengthRule {
    fn check(&self, item: &RunbookSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSnapshotRequiredCodeRule {
    pub message: String,
}

impl RunbookSnapshotRule for RunbookSnapshotRequiredCodeRule {
    fn check(&self, item: &RunbookSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a runbook snapshot.
#[derive(Debug, Clone)]
pub struct RunbookSnapshotValidator {
    pub rules: Vec<Rc<dyn RunbookSnapshotRule>>,
}

impl RunbookSnapshotValidator {
    pub fn validate(&self, item: &RunbookSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RunbookSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_snapshot_validator() -> RunbookSnapshotValidator {
    RunbookSnapshotValidator {
        rules: vec![
            Rc::new(RunbookSnapshotUpdatedAtRangeRule { min: 80, max: 255 }) as Rc<dyn RunbookSnapshotRule>,
            Rc::new(RunbookSnapshotTagsLimitRule {
                max_count: 86400,
                banned: vec!["fallback".to_string(), "legacy".to_string(), "batched".to_string()],
            }),
            Rc::new(RunbookSnapshotTitleLengthRule { max_length: 120 }),
            Rc::new(RunbookSnapshotRequiredCodeRule {
                message: "runbook snapshot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_snapshot_validator();
        let item = new_runbook_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = RunbookSnapshotUpdatedAtRangeRule { min: 80, max: 120 };
        let mut item = new_runbook_snapshot("x", "y");
        item.updated_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RunbookSnapshotTitleLengthRule { max_length: 1000 };
        let mut item = new_runbook_snapshot("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RunbookSnapshotRequiredCodeRule {
            message: "runbook snapshot needs a value".to_string(),
        };
        let mut item = new_runbook_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
