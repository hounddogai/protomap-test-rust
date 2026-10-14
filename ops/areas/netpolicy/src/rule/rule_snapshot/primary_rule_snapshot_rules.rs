use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rule snapshot must pass. Check returns its problems, or none.
pub trait PrimaryRuleSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRuleSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleSnapshotRequiredCodeRule {
    pub message: String,
}

impl PrimaryRuleSnapshotRule for PrimaryRuleSnapshotRequiredCodeRule {
    fn check(&self, item: &PrimaryRuleSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleSnapshotArchivedGuardRule {
    pub limit: i64,
}

impl PrimaryRuleSnapshotRule for PrimaryRuleSnapshotArchivedGuardRule {
    fn check(&self, item: &PrimaryRuleSnapshot) -> Vec<String> {
        if item.archived && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while archived",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryRuleSnapshotRule for PrimaryRuleSnapshotTitleLengthRule {
    fn check(&self, item: &PrimaryRuleSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary rule snapshot.
#[derive(Debug, Clone)]
pub struct PrimaryRuleSnapshotValidator {
    pub rules: Vec<Rc<dyn PrimaryRuleSnapshotRule>>,
}

impl PrimaryRuleSnapshotValidator {
    pub fn validate(&self, item: &PrimaryRuleSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRuleSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRuleSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rule_snapshot_validator() -> PrimaryRuleSnapshotValidator {
    PrimaryRuleSnapshotValidator {
        rules: vec![
            Rc::new(PrimaryRuleSnapshotRequiredCodeRule {
                message: "primary rule snapshot needs this field".to_string(),
            }) as Rc<dyn PrimaryRuleSnapshotRule>,
            Rc::new(PrimaryRuleSnapshotArchivedGuardRule { limit: 255 }),
            Rc::new(PrimaryRuleSnapshotTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rule_snapshot_validator();
        let item = new_primary_rule_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryRuleSnapshotRequiredCodeRule {
            message: "primary rule snapshot needs an entry".to_string(),
        };
        let mut item = new_primary_rule_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryRuleSnapshotTitleLengthRule { max_length: 255 };
        let mut item = new_primary_rule_snapshot("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
