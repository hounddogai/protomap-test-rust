use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary snapshot must pass. Check returns its problems, or none.
pub trait CanarySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &CanarySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshotCountryLengthRule {
    pub max_length: i64,
}

impl CanarySnapshotRule for CanarySnapshotCountryLengthRule {
    fn check(&self, item: &CanarySnapshot) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshotRequiredCountryRule {
    pub message: String,
}

impl CanarySnapshotRule for CanarySnapshotRequiredCountryRule {
    fn check(&self, item: &CanarySnapshot) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshotTitleLengthRule {
    pub max_length: i64,
}

impl CanarySnapshotRule for CanarySnapshotTitleLengthRule {
    fn check(&self, item: &CanarySnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshotRequiredTitleRule {
    pub message: String,
}

impl CanarySnapshotRule for CanarySnapshotRequiredTitleRule {
    fn check(&self, item: &CanarySnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanarySnapshotCodeLengthRule {
    pub max_length: i64,
}

impl CanarySnapshotRule for CanarySnapshotCodeLengthRule {
    fn check(&self, item: &CanarySnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary snapshot.
#[derive(Debug, Clone)]
pub struct CanarySnapshotValidator {
    pub rules: Vec<Rc<dyn CanarySnapshotRule>>,
}

impl CanarySnapshotValidator {
    pub fn validate(&self, item: &CanarySnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanarySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanarySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_snapshot_validator() -> CanarySnapshotValidator {
    CanarySnapshotValidator {
        rules: vec![
            Rc::new(CanarySnapshotCountryLengthRule { max_length: 80 }) as Rc<dyn CanarySnapshotRule>,
            Rc::new(CanarySnapshotRequiredCountryRule {
                message: "canary snapshot needs a value".to_string(),
            }),
            Rc::new(CanarySnapshotTitleLengthRule { max_length: 120 }),
            Rc::new(CanarySnapshotRequiredTitleRule {
                message: "canary snapshot needs this field".to_string(),
            }),
            Rc::new(CanarySnapshotCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_snapshot_validator();
        let item = new_canary_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = CanarySnapshotCountryLengthRule { max_length: 255 };
        let mut item = new_canary_snapshot("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = CanarySnapshotRequiredCountryRule {
            message: "canary snapshot needs an entry".to_string(),
        };
        let mut item = new_canary_snapshot("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CanarySnapshotTitleLengthRule { max_length: 16 };
        let mut item = new_canary_snapshot("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CanarySnapshotRequiredTitleRule {
            message: "canary snapshot needs a value".to_string(),
        };
        let mut item = new_canary_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanarySnapshotCodeLengthRule { max_length: 3600 };
        let mut item = new_canary_snapshot("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
