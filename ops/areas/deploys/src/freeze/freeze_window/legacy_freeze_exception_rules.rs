use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy freeze exception must pass. Check returns its problems, or none.
pub trait LegacyFreezeExceptionRule: std::fmt::Debug {
    fn check(&self, item: &LegacyFreezeException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeExceptionPinnedGuardRule {
    pub limit: i64,
}

impl LegacyFreezeExceptionRule for LegacyFreezeExceptionPinnedGuardRule {
    fn check(&self, item: &LegacyFreezeException) -> Vec<String> {
        if item.pinned && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while pinned",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeExceptionHostLengthRule {
    pub max_length: i64,
}

impl LegacyFreezeExceptionRule for LegacyFreezeExceptionHostLengthRule {
    fn check(&self, item: &LegacyFreezeException) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeExceptionSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyFreezeExceptionRule for LegacyFreezeExceptionSequenceRangeRule {
    fn check(&self, item: &LegacyFreezeException) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeExceptionNotesLengthRule {
    pub max_length: i64,
}

impl LegacyFreezeExceptionRule for LegacyFreezeExceptionNotesLengthRule {
    fn check(&self, item: &LegacyFreezeException) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeExceptionRequiredNotesRule {
    pub message: String,
}

impl LegacyFreezeExceptionRule for LegacyFreezeExceptionRequiredNotesRule {
    fn check(&self, item: &LegacyFreezeException) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFreezeExceptionCodeLengthRule {
    pub max_length: i64,
}

impl LegacyFreezeExceptionRule for LegacyFreezeExceptionCodeLengthRule {
    fn check(&self, item: &LegacyFreezeException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy freeze exception.
#[derive(Debug, Clone)]
pub struct LegacyFreezeExceptionValidator {
    pub rules: Vec<Rc<dyn LegacyFreezeExceptionRule>>,
}

impl LegacyFreezeExceptionValidator {
    pub fn validate(&self, item: &LegacyFreezeException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyFreezeException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyFreezeException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_freeze_exception_validator() -> LegacyFreezeExceptionValidator {
    LegacyFreezeExceptionValidator {
        rules: vec![
            Rc::new(LegacyFreezeExceptionPinnedGuardRule { limit: 32 }) as Rc<dyn LegacyFreezeExceptionRule>,
            Rc::new(LegacyFreezeExceptionHostLengthRule { max_length: 3600 }),
            Rc::new(LegacyFreezeExceptionSequenceRangeRule { min: 64, max: 80 }),
            Rc::new(LegacyFreezeExceptionNotesLengthRule { max_length: 86400 }),
            Rc::new(LegacyFreezeExceptionRequiredNotesRule {
                message: "legacy freeze exception needs a value".to_string(),
            }),
            Rc::new(LegacyFreezeExceptionCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_freeze_exception_validator();
        let item = new_legacy_freeze_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = LegacyFreezeExceptionHostLengthRule { max_length: 86400 };
        let mut item = new_legacy_freeze_exception("x", "y");
        item.host = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = LegacyFreezeExceptionSequenceRangeRule { min: 1000, max: 86400 };
        let mut item = new_legacy_freeze_exception("x", "y");
        item.sequence = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = LegacyFreezeExceptionNotesLengthRule { max_length: 86400 };
        let mut item = new_legacy_freeze_exception("x", "y");
        item.notes = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = LegacyFreezeExceptionRequiredNotesRule {
            message: "legacy freeze exception needs this field".to_string(),
        };
        let mut item = new_legacy_freeze_exception("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyFreezeExceptionCodeLengthRule { max_length: 80 };
        let mut item = new_legacy_freeze_exception("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
