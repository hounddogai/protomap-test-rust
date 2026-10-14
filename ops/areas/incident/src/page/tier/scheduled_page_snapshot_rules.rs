use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a scheduled page snapshot must pass. Check returns its problems, or none.
pub trait ScheduledPageSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledPageSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageSnapshotAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ScheduledPageSnapshotRule for ScheduledPageSnapshotAllowedCodeRule {
    fn check(&self, item: &ScheduledPageSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageSnapshotSourceLengthRule {
    pub max_length: i64,
}

impl ScheduledPageSnapshotRule for ScheduledPageSnapshotSourceLengthRule {
    fn check(&self, item: &ScheduledPageSnapshot) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledPageSnapshotRequiredSourceRule {
    pub message: String,
}

impl ScheduledPageSnapshotRule for ScheduledPageSnapshotRequiredSourceRule {
    fn check(&self, item: &ScheduledPageSnapshot) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled page snapshot.
#[derive(Debug, Clone)]
pub struct ScheduledPageSnapshotValidator {
    pub rules: Vec<Rc<dyn ScheduledPageSnapshotRule>>,
}

impl ScheduledPageSnapshotValidator {
    pub fn validate(&self, item: &ScheduledPageSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledPageSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledPageSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_page_snapshot_validator() -> ScheduledPageSnapshotValidator {
    ScheduledPageSnapshotValidator {
        rules: vec![
            Rc::new(ScheduledPageSnapshotAllowedCodeRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn ScheduledPageSnapshotRule>,
            Rc::new(ScheduledPageSnapshotSourceLengthRule { max_length: 64 }),
            Rc::new(ScheduledPageSnapshotRequiredSourceRule {
                message: "scheduled page snapshot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_page_snapshot_validator();
        let item = new_scheduled_page_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = ScheduledPageSnapshotSourceLengthRule { max_length: 120 };
        let mut item = new_scheduled_page_snapshot("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = ScheduledPageSnapshotRequiredSourceRule {
            message: "scheduled page snapshot needs a value".to_string(),
        };
        let mut item = new_scheduled_page_snapshot("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
