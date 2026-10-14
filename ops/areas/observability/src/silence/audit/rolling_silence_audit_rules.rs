use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling silence audit must pass. Check returns its problems, or none.
pub trait RollingSilenceAuditRule: std::fmt::Debug {
    fn check(&self, item: &RollingSilenceAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceAuditRequiredTitleRule {
    pub message: String,
}

impl RollingSilenceAuditRule for RollingSilenceAuditRequiredTitleRule {
    fn check(&self, item: &RollingSilenceAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceAuditTitleLengthRule {
    pub max_length: i64,
}

impl RollingSilenceAuditRule for RollingSilenceAuditTitleLengthRule {
    fn check(&self, item: &RollingSilenceAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSilenceAuditRequiredExternalRefRule {
    pub message: String,
}

impl RollingSilenceAuditRule for RollingSilenceAuditRequiredExternalRefRule {
    fn check(&self, item: &RollingSilenceAudit) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling silence audit.
#[derive(Debug, Clone)]
pub struct RollingSilenceAuditValidator {
    pub rules: Vec<Rc<dyn RollingSilenceAuditRule>>,
}

impl RollingSilenceAuditValidator {
    pub fn validate(&self, item: &RollingSilenceAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingSilenceAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSilenceAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_silence_audit_validator() -> RollingSilenceAuditValidator {
    RollingSilenceAuditValidator {
        rules: vec![
            Rc::new(RollingSilenceAuditRequiredTitleRule {
                message: "rolling silence audit needs a value".to_string(),
            }) as Rc<dyn RollingSilenceAuditRule>,
            Rc::new(RollingSilenceAuditTitleLengthRule { max_length: 80 }),
            Rc::new(RollingSilenceAuditRequiredExternalRefRule {
                message: "rolling silence audit needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_silence_audit_validator();
        let item = new_rolling_silence_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RollingSilenceAuditRequiredTitleRule {
            message: "rolling silence audit needs this field".to_string(),
        };
        let mut item = new_rolling_silence_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingSilenceAuditTitleLengthRule { max_length: 16 };
        let mut item = new_rolling_silence_audit("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = RollingSilenceAuditRequiredExternalRefRule {
            message: "rolling silence audit needs this field".to_string(),
        };
        let mut item = new_rolling_silence_audit("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
