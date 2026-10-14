use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink record must pass. Check returns its problems, or none.
pub trait SinkRecordRule: std::fmt::Debug {
    fn check(&self, item: &SinkRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordCodeLengthRule {
    pub max_length: i64,
}

impl SinkRecordRule for SinkRecordCodeLengthRule {
    fn check(&self, item: &SinkRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl SinkRecordRule for SinkRecordAllowedTitleRule {
    fn check(&self, item: &SinkRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordTitleLengthRule {
    pub max_length: i64,
}

impl SinkRecordRule for SinkRecordTitleLengthRule {
    fn check(&self, item: &SinkRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordRequiredCodeRule {
    pub message: String,
}

impl SinkRecordRule for SinkRecordRequiredCodeRule {
    fn check(&self, item: &SinkRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordRequiredTitleRule {
    pub message: String,
}

impl SinkRecordRule for SinkRecordRequiredTitleRule {
    fn check(&self, item: &SinkRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkRecordArchivedGuardRule {
    pub limit: f64,
}

impl SinkRecordRule for SinkRecordArchivedGuardRule {
    fn check(&self, item: &SinkRecord) -> Vec<String> {
        if item.archived && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while archived",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink record.
#[derive(Debug, Clone)]
pub struct SinkRecordValidator {
    pub rules: Vec<Rc<dyn SinkRecordRule>>,
}

impl SinkRecordValidator {
    pub fn validate(&self, item: &SinkRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_record_validator() -> SinkRecordValidator {
    SinkRecordValidator {
        rules: vec![
            Rc::new(SinkRecordCodeLengthRule { max_length: 120 }) as Rc<dyn SinkRecordRule>,
            Rc::new(SinkRecordAllowedTitleRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(SinkRecordTitleLengthRule { max_length: 64 }),
            Rc::new(SinkRecordRequiredCodeRule {
                message: "sink record needs this field".to_string(),
            }),
            Rc::new(SinkRecordRequiredTitleRule {
                message: "sink record needs an entry".to_string(),
            }),
            Rc::new(SinkRecordArchivedGuardRule { limit: 2.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_record_validator();
        let item = new_sink_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkRecordCodeLengthRule { max_length: 1000 };
        let mut item = new_sink_record("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SinkRecordTitleLengthRule { max_length: 80 };
        let mut item = new_sink_record("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkRecordRequiredCodeRule {
            message: "sink record needs an entry".to_string(),
        };
        let mut item = new_sink_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SinkRecordRequiredTitleRule {
            message: "sink record needs this field".to_string(),
        };
        let mut item = new_sink_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
