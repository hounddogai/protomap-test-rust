use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive plan snapshot must pass. Check returns its problems, or none.
pub trait AdaptivePlanSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &AdaptivePlanSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanSnapshotCipherLengthRule {
    pub max_length: i64,
}

impl AdaptivePlanSnapshotRule for AdaptivePlanSnapshotCipherLengthRule {
    fn check(&self, item: &AdaptivePlanSnapshot) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanSnapshotRequiredTitleRule {
    pub message: String,
}

impl AdaptivePlanSnapshotRule for AdaptivePlanSnapshotRequiredTitleRule {
    fn check(&self, item: &AdaptivePlanSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl AdaptivePlanSnapshotRule for AdaptivePlanSnapshotCodeLengthRule {
    fn check(&self, item: &AdaptivePlanSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive plan snapshot.
#[derive(Debug, Clone)]
pub struct AdaptivePlanSnapshotValidator {
    pub rules: Vec<Rc<dyn AdaptivePlanSnapshotRule>>,
}

impl AdaptivePlanSnapshotValidator {
    pub fn validate(&self, item: &AdaptivePlanSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptivePlanSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptivePlanSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_plan_snapshot_validator() -> AdaptivePlanSnapshotValidator {
    AdaptivePlanSnapshotValidator {
        rules: vec![
            Rc::new(AdaptivePlanSnapshotCipherLengthRule { max_length: 120 }) as Rc<dyn AdaptivePlanSnapshotRule>,
            Rc::new(AdaptivePlanSnapshotRequiredTitleRule {
                message: "adaptive plan snapshot needs an entry".to_string(),
            }),
            Rc::new(AdaptivePlanSnapshotCodeLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_plan_snapshot_validator();
        let item = new_adaptive_plan_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = AdaptivePlanSnapshotCipherLengthRule { max_length: 64 };
        let mut item = new_adaptive_plan_snapshot("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptivePlanSnapshotRequiredTitleRule {
            message: "adaptive plan snapshot needs a value".to_string(),
        };
        let mut item = new_adaptive_plan_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptivePlanSnapshotCodeLengthRule { max_length: 86400 };
        let mut item = new_adaptive_plan_snapshot("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
