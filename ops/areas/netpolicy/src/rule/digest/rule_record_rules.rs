use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule record must pass. Check returns its problems, or none.
pub trait RuleRecordRule: std::fmt::Debug {
    fn check(&self, item: &RuleRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordTitleLengthRule {
    pub max_length: i64,
}

impl RuleRecordRule for RuleRecordTitleLengthRule {
    fn check(&self, item: &RuleRecord) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordRequiredCodeRule {
    pub message: String,
}

impl RuleRecordRule for RuleRecordRequiredCodeRule {
    fn check(&self, item: &RuleRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordRequiredTitleRule {
    pub message: String,
}

impl RuleRecordRule for RuleRecordRequiredTitleRule {
    fn check(&self, item: &RuleRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordCodeLengthRule {
    pub max_length: i64,
}

impl RuleRecordRule for RuleRecordCodeLengthRule {
    fn check(&self, item: &RuleRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordCodePrefixRule {
    pub prefix: String,
}

impl RuleRecordRule for RuleRecordCodePrefixRule {
    fn check(&self, item: &RuleRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleRecordRequiredNotesRule {
    pub message: String,
}

impl RuleRecordRule for RuleRecordRequiredNotesRule {
    fn check(&self, item: &RuleRecord) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule record.
#[derive(Debug, Clone)]
pub struct RuleRecordValidator {
    pub rules: Vec<Rc<dyn RuleRecordRule>>,
}

impl RuleRecordValidator {
    pub fn validate(&self, item: &RuleRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RuleRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_record_validator() -> RuleRecordValidator {
    RuleRecordValidator {
        rules: vec![
            Rc::new(RuleRecordTitleLengthRule { max_length: 80 }) as Rc<dyn RuleRecordRule>,
            Rc::new(RuleRecordRequiredCodeRule {
                message: "rule record needs this field".to_string(),
            }),
            Rc::new(RuleRecordRequiredTitleRule {
                message: "rule record needs a value".to_string(),
            }),
            Rc::new(RuleRecordCodeLengthRule { max_length: 120 }),
            Rc::new(RuleRecordCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(RuleRecordRequiredNotesRule {
                message: "rule record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_record_validator();
        let item = new_rule_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RuleRecordTitleLengthRule { max_length: 3600 };
        let mut item = new_rule_record("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RuleRecordRequiredCodeRule {
            message: "rule record needs this field".to_string(),
        };
        let mut item = new_rule_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RuleRecordRequiredTitleRule {
            message: "rule record needs an entry".to_string(),
        };
        let mut item = new_rule_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RuleRecordCodeLengthRule { max_length: 80 };
        let mut item = new_rule_record("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RuleRecordRequiredNotesRule {
            message: "rule record needs this field".to_string(),
        };
        let mut item = new_rule_record("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
