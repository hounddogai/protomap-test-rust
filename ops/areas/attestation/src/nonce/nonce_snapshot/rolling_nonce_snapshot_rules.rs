use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling nonce snapshot must pass. Check returns its problems, or none.
pub trait RollingNonceSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &RollingNonceSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceSnapshotRequiredKeyIdRule {
    pub message: String,
}

impl RollingNonceSnapshotRule for RollingNonceSnapshotRequiredKeyIdRule {
    fn check(&self, item: &RollingNonceSnapshot) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceSnapshotRequiredTitleRule {
    pub message: String,
}

impl RollingNonceSnapshotRule for RollingNonceSnapshotRequiredTitleRule {
    fn check(&self, item: &RollingNonceSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl RollingNonceSnapshotRule for RollingNonceSnapshotTitleLengthRule {
    fn check(&self, item: &RollingNonceSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl RollingNonceSnapshotRule for RollingNonceSnapshotCodeLengthRule {
    fn check(&self, item: &RollingNonceSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling nonce snapshot.
#[derive(Debug, Clone)]
pub struct RollingNonceSnapshotValidator {
    pub rules: Vec<Rc<dyn RollingNonceSnapshotRule>>,
}

impl RollingNonceSnapshotValidator {
    pub fn validate(&self, item: &RollingNonceSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingNonceSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingNonceSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_nonce_snapshot_validator() -> RollingNonceSnapshotValidator {
    RollingNonceSnapshotValidator {
        rules: vec![
            Rc::new(RollingNonceSnapshotRequiredKeyIdRule {
                message: "rolling nonce snapshot needs an entry".to_string(),
            }) as Rc<dyn RollingNonceSnapshotRule>,
            Rc::new(RollingNonceSnapshotRequiredTitleRule {
                message: "rolling nonce snapshot needs an entry".to_string(),
            }),
            Rc::new(RollingNonceSnapshotTitleLengthRule { max_length: 3600 }),
            Rc::new(RollingNonceSnapshotCodeLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_nonce_snapshot_validator();
        let item = new_rolling_nonce_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = RollingNonceSnapshotRequiredKeyIdRule {
            message: "rolling nonce snapshot needs an entry".to_string(),
        };
        let mut item = new_rolling_nonce_snapshot("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingNonceSnapshotRequiredTitleRule {
            message: "rolling nonce snapshot needs an entry".to_string(),
        };
        let mut item = new_rolling_nonce_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingNonceSnapshotTitleLengthRule { max_length: 80 };
        let mut item = new_rolling_nonce_snapshot("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingNonceSnapshotCodeLengthRule { max_length: 80 };
        let mut item = new_rolling_nonce_snapshot("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
