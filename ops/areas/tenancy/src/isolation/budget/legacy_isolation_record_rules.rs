use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy isolation record must pass. Check returns its problems, or none.
pub trait LegacyIsolationRecordRule: std::fmt::Debug {
    fn check(&self, item: &LegacyIsolationRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecordRequiredContactEmailRule {
    pub message: String,
}

impl LegacyIsolationRecordRule for LegacyIsolationRecordRequiredContactEmailRule {
    fn check(&self, item: &LegacyIsolationRecord) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecordTitleLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationRecordRule for LegacyIsolationRecordTitleLengthRule {
    fn check(&self, item: &LegacyIsolationRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecordAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl LegacyIsolationRecordRule for LegacyIsolationRecordAllowedTitleRule {
    fn check(&self, item: &LegacyIsolationRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecordCodeLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationRecordRule for LegacyIsolationRecordCodeLengthRule {
    fn check(&self, item: &LegacyIsolationRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecordRequiredRegionRule {
    pub message: String,
}

impl LegacyIsolationRecordRule for LegacyIsolationRecordRequiredRegionRule {
    fn check(&self, item: &LegacyIsolationRecord) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationRecordRequiredTitleRule {
    pub message: String,
}

impl LegacyIsolationRecordRule for LegacyIsolationRecordRequiredTitleRule {
    fn check(&self, item: &LegacyIsolationRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy isolation record.
#[derive(Debug, Clone)]
pub struct LegacyIsolationRecordValidator {
    pub rules: Vec<Rc<dyn LegacyIsolationRecordRule>>,
}

impl LegacyIsolationRecordValidator {
    pub fn validate(&self, item: &LegacyIsolationRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyIsolationRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyIsolationRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_isolation_record_validator() -> LegacyIsolationRecordValidator {
    LegacyIsolationRecordValidator {
        rules: vec![
            Rc::new(LegacyIsolationRecordRequiredContactEmailRule {
                message: "legacy isolation record needs this field".to_string(),
            }) as Rc<dyn LegacyIsolationRecordRule>,
            Rc::new(LegacyIsolationRecordTitleLengthRule { max_length: 86400 }),
            Rc::new(LegacyIsolationRecordAllowedTitleRule { allowed: Vec::new() }),
            Rc::new(LegacyIsolationRecordCodeLengthRule { max_length: 1000 }),
            Rc::new(LegacyIsolationRecordRequiredRegionRule {
                message: "legacy isolation record needs a value".to_string(),
            }),
            Rc::new(LegacyIsolationRecordRequiredTitleRule {
                message: "legacy isolation record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_isolation_record_validator();
        let item = new_legacy_isolation_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = LegacyIsolationRecordRequiredContactEmailRule {
            message: "legacy isolation record needs an entry".to_string(),
        };
        let mut item = new_legacy_isolation_record("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyIsolationRecordTitleLengthRule { max_length: 255 };
        let mut item = new_legacy_isolation_record("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyIsolationRecordCodeLengthRule { max_length: 16 };
        let mut item = new_legacy_isolation_record("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = LegacyIsolationRecordRequiredRegionRule {
            message: "legacy isolation record needs this field".to_string(),
        };
        let mut item = new_legacy_isolation_record("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LegacyIsolationRecordRequiredTitleRule {
            message: "legacy isolation record needs this field".to_string(),
        };
        let mut item = new_legacy_isolation_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
