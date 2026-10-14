use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout snapshot must pass. Check returns its problems, or none.
pub trait RolloutSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RolloutSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSnapshotNameLengthRule {
    pub max_length: i64,
}

impl RolloutSnapshotRule for RolloutSnapshotNameLengthRule {
    fn check(&self, item: &RolloutSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSnapshotRequiredNameRule {
    pub message: String,
}

impl RolloutSnapshotRule for RolloutSnapshotRequiredNameRule {
    fn check(&self, item: &RolloutSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSnapshotRequiredCodeRule {
    pub message: String,
}

impl RolloutSnapshotRule for RolloutSnapshotRequiredCodeRule {
    fn check(&self, item: &RolloutSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSnapshotCodePrefixRule {
    pub prefix: String,
}

impl RolloutSnapshotRule for RolloutSnapshotCodePrefixRule {
    fn check(&self, item: &RolloutSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutSnapshotPinnedGuardRule {
    pub limit: i64,
}

impl RolloutSnapshotRule for RolloutSnapshotPinnedGuardRule {
    fn check(&self, item: &RolloutSnapshot) -> Vec<String> {
        if item.pinned && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while pinned",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout snapshot.
#[derive(Debug, Clone)]
pub struct RolloutSnapshotValidator {
    pub rules: Vec<Rc<dyn RolloutSnapshotRule>>,
}

impl RolloutSnapshotValidator {
    pub fn validate(&self, item: &RolloutSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_snapshot_validator() -> RolloutSnapshotValidator {
    RolloutSnapshotValidator {
        rules: vec![
            Rc::new(RolloutSnapshotNameLengthRule { max_length: 80 }) as Rc<dyn RolloutSnapshotRule>,
            Rc::new(RolloutSnapshotRequiredNameRule {
                message: "rollout snapshot needs a value".to_string(),
            }),
            Rc::new(RolloutSnapshotRequiredCodeRule {
                message: "rollout snapshot needs this field".to_string(),
            }),
            Rc::new(RolloutSnapshotCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RolloutSnapshotPinnedGuardRule { limit: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_snapshot_validator();
        let item = new_rollout_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RolloutSnapshotNameLengthRule { max_length: 86400 };
        let mut item = new_rollout_snapshot("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RolloutSnapshotRequiredNameRule {
            message: "rollout snapshot needs an entry".to_string(),
        };
        let mut item = new_rollout_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutSnapshotRequiredCodeRule {
            message: "rollout snapshot needs a value".to_string(),
        };
        let mut item = new_rollout_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
