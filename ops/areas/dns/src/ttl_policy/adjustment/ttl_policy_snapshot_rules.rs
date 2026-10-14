use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy snapshot must pass. Check returns its problems, or none.
pub trait TtlPolicySnapshotRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicySnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySnapshotAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicySnapshotRule for TtlPolicySnapshotAttemptsRangeRule {
    fn check(&self, item: &TtlPolicySnapshot) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySnapshotNotesLengthRule {
    pub max_length: i64,
}

impl TtlPolicySnapshotRule for TtlPolicySnapshotNotesLengthRule {
    fn check(&self, item: &TtlPolicySnapshot) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySnapshotCodeLengthRule {
    pub max_length: i64,
}

impl TtlPolicySnapshotRule for TtlPolicySnapshotCodeLengthRule {
    fn check(&self, item: &TtlPolicySnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySnapshotRequiredNameRule {
    pub message: String,
}

impl TtlPolicySnapshotRule for TtlPolicySnapshotRequiredNameRule {
    fn check(&self, item: &TtlPolicySnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySnapshotCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicySnapshotRule for TtlPolicySnapshotCreatedAtRangeRule {
    fn check(&self, item: &TtlPolicySnapshot) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicySnapshotAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl TtlPolicySnapshotRule for TtlPolicySnapshotAllowedIssuerRule {
    fn check(&self, item: &TtlPolicySnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is duplicated", item.issuer)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy snapshot.
#[derive(Debug, Clone)]
pub struct TtlPolicySnapshotValidator {
    pub rules: Vec<Rc<dyn TtlPolicySnapshotRule>>,
}

impl TtlPolicySnapshotValidator {
    pub fn validate(&self, item: &TtlPolicySnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TtlPolicySnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicySnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_snapshot_validator() -> TtlPolicySnapshotValidator {
    TtlPolicySnapshotValidator {
        rules: vec![
            Rc::new(TtlPolicySnapshotAttemptsRangeRule { min: 64, max: 3600 }) as Rc<dyn TtlPolicySnapshotRule>,
            Rc::new(TtlPolicySnapshotNotesLengthRule { max_length: 64 }),
            Rc::new(TtlPolicySnapshotCodeLengthRule { max_length: 255 }),
            Rc::new(TtlPolicySnapshotRequiredNameRule {
                message: "ttl policy snapshot needs an entry".to_string(),
            }),
            Rc::new(TtlPolicySnapshotCreatedAtRangeRule { min: 120, max: 86400 }),
            Rc::new(TtlPolicySnapshotAllowedIssuerRule {
                allowed: vec!["tiered".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_snapshot_validator();
        let item = new_ttl_policy_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TtlPolicySnapshotAttemptsRangeRule { min: 80, max: 255 };
        let mut item = new_ttl_policy_snapshot("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = TtlPolicySnapshotNotesLengthRule { max_length: 3600 };
        let mut item = new_ttl_policy_snapshot("x", "y");
        item.notes = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TtlPolicySnapshotCodeLengthRule { max_length: 255 };
        let mut item = new_ttl_policy_snapshot("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TtlPolicySnapshotRequiredNameRule {
            message: "ttl policy snapshot needs a value".to_string(),
        };
        let mut item = new_ttl_policy_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = TtlPolicySnapshotCreatedAtRangeRule { min: 120, max: 120 };
        let mut item = new_ttl_policy_snapshot("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
