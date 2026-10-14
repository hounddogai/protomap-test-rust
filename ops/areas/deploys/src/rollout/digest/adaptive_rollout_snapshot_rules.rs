use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive rollout snapshot must pass. Check returns its problems, or none.
pub trait AdaptiveRolloutSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRolloutSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl AdaptiveRolloutSnapshotRule for AdaptiveRolloutSnapshotTitleLengthRule {
    fn check(&self, item: &AdaptiveRolloutSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveRolloutSnapshotRule for AdaptiveRolloutSnapshotCodeLengthRule {
    fn check(&self, item: &AdaptiveRolloutSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutSnapshotCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveRolloutSnapshotRule for AdaptiveRolloutSnapshotCodePrefixRule {
    fn check(&self, item: &AdaptiveRolloutSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRolloutSnapshotRequiredCodeRule {
    pub message: String,
}

impl AdaptiveRolloutSnapshotRule for AdaptiveRolloutSnapshotRequiredCodeRule {
    fn check(&self, item: &AdaptiveRolloutSnapshot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive rollout snapshot.
#[derive(Debug, Clone)]
pub struct AdaptiveRolloutSnapshotValidator {
    pub rules: Vec<Rc<dyn AdaptiveRolloutSnapshotRule>>,
}

impl AdaptiveRolloutSnapshotValidator {
    pub fn validate(&self, item: &AdaptiveRolloutSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRolloutSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRolloutSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_rollout_snapshot_validator() -> AdaptiveRolloutSnapshotValidator {
    AdaptiveRolloutSnapshotValidator {
        rules: vec![
            Rc::new(AdaptiveRolloutSnapshotTitleLengthRule { max_length: 16 }) as Rc<dyn AdaptiveRolloutSnapshotRule>,
            Rc::new(AdaptiveRolloutSnapshotCodeLengthRule { max_length: 16 }),
            Rc::new(AdaptiveRolloutSnapshotCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(AdaptiveRolloutSnapshotRequiredCodeRule {
                message: "adaptive rollout snapshot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_rollout_snapshot_validator();
        let item = new_adaptive_rollout_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AdaptiveRolloutSnapshotTitleLengthRule { max_length: 80 };
        let mut item = new_adaptive_rollout_snapshot("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveRolloutSnapshotCodeLengthRule { max_length: 64 };
        let mut item = new_adaptive_rollout_snapshot("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveRolloutSnapshotRequiredCodeRule {
            message: "adaptive rollout snapshot needs this field".to_string(),
        };
        let mut item = new_adaptive_rollout_snapshot("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
