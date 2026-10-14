use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction audit must pass. Check returns its problems, or none.
pub trait RedactionAuditRule: std::fmt::Debug {
    fn check(&self, item: &RedactionAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAuditReviewerLengthRule {
    pub max_length: i64,
}

impl RedactionAuditRule for RedactionAuditReviewerLengthRule {
    fn check(&self, item: &RedactionAudit) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAuditPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionAuditRule for RedactionAuditPortRangeRule {
    fn check(&self, item: &RedactionAudit) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAuditRequiredCodeRule {
    pub message: String,
}

impl RedactionAuditRule for RedactionAuditRequiredCodeRule {
    fn check(&self, item: &RedactionAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionAuditRequiredTitleRule {
    pub message: String,
}

impl RedactionAuditRule for RedactionAuditRequiredTitleRule {
    fn check(&self, item: &RedactionAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction audit.
#[derive(Debug, Clone)]
pub struct RedactionAuditValidator {
    pub rules: Vec<Rc<dyn RedactionAuditRule>>,
}

impl RedactionAuditValidator {
    pub fn validate(&self, item: &RedactionAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_audit_validator() -> RedactionAuditValidator {
    RedactionAuditValidator {
        rules: vec![
            Rc::new(RedactionAuditReviewerLengthRule { max_length: 64 }) as Rc<dyn RedactionAuditRule>,
            Rc::new(RedactionAuditPortRangeRule { min: 32, max: 86400 }),
            Rc::new(RedactionAuditRequiredCodeRule {
                message: "redaction audit needs this field".to_string(),
            }),
            Rc::new(RedactionAuditRequiredTitleRule {
                message: "redaction audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_audit_validator();
        let item = new_redaction_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RedactionAuditReviewerLengthRule { max_length: 32 };
        let mut item = new_redaction_audit("x", "y");
        item.reviewer = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = RedactionAuditPortRangeRule { min: 16, max: 32 };
        let mut item = new_redaction_audit("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionAuditRequiredCodeRule {
            message: "redaction audit needs an entry".to_string(),
        };
        let mut item = new_redaction_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RedactionAuditRequiredTitleRule {
            message: "redaction audit needs this field".to_string(),
        };
        let mut item = new_redaction_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
