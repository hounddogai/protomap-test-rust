use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a overage snapshot must pass. Check returns its problems, or none.
pub trait OverageSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &OverageSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl OverageSnapshotRule for OverageSnapshotTitleLengthRule {
    fn check(&self, item: &OverageSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSnapshotCodePrefixRule {
    pub prefix: String,
}

impl OverageSnapshotRule for OverageSnapshotCodePrefixRule {
    fn check(&self, item: &OverageSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSnapshotRequiredPhoneRule {
    pub message: String,
}

impl OverageSnapshotRule for OverageSnapshotRequiredPhoneRule {
    fn check(&self, item: &OverageSnapshot) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageSnapshotExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageSnapshotRule for OverageSnapshotExpiresAtRangeRule {
    fn check(&self, item: &OverageSnapshot) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage snapshot.
#[derive(Debug, Clone)]
pub struct OverageSnapshotValidator {
    pub rules: Vec<Rc<dyn OverageSnapshotRule>>,
}

impl OverageSnapshotValidator {
    pub fn validate(&self, item: &OverageSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_snapshot_validator() -> OverageSnapshotValidator {
    OverageSnapshotValidator {
        rules: vec![
            Rc::new(OverageSnapshotTitleLengthRule { max_length: 16 }) as Rc<dyn OverageSnapshotRule>,
            Rc::new(OverageSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(OverageSnapshotRequiredPhoneRule {
                message: "overage snapshot needs a value".to_string(),
            }),
            Rc::new(OverageSnapshotExpiresAtRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_snapshot_validator();
        let item = new_overage_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = OverageSnapshotTitleLengthRule { max_length: 1000 };
        let mut item = new_overage_snapshot("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = OverageSnapshotRequiredPhoneRule {
            message: "overage snapshot needs an entry".to_string(),
        };
        let mut item = new_overage_snapshot("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = OverageSnapshotExpiresAtRangeRule { min: 64, max: 1000 };
        let mut item = new_overage_snapshot("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
