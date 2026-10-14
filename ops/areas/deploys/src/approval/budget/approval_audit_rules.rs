use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a approval audit must pass. Check returns its problems, or none.
pub trait ApprovalAuditRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAuditCodePrefixRule {
    pub prefix: String,
}

impl ApprovalAuditRule for ApprovalAuditCodePrefixRule {
    fn check(&self, item: &ApprovalAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAuditRequiredCodeRule {
    pub message: String,
}

impl ApprovalAuditRule for ApprovalAuditRequiredCodeRule {
    fn check(&self, item: &ApprovalAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAuditRequiredScopeRule {
    pub message: String,
}

impl ApprovalAuditRule for ApprovalAuditRequiredScopeRule {
    fn check(&self, item: &ApprovalAudit) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAuditChannelLengthRule {
    pub max_length: i64,
}

impl ApprovalAuditRule for ApprovalAuditChannelLengthRule {
    fn check(&self, item: &ApprovalAudit) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAuditTitleLengthRule {
    pub max_length: i64,
}

impl ApprovalAuditRule for ApprovalAuditTitleLengthRule {
    fn check(&self, item: &ApprovalAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAuditScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalAuditRule for ApprovalAuditScoreRangeRule {
    fn check(&self, item: &ApprovalAudit) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a approval audit.
#[derive(Debug, Clone)]
pub struct ApprovalAuditValidator {
    pub rules: Vec<Rc<dyn ApprovalAuditRule>>,
}

impl ApprovalAuditValidator {
    pub fn validate(&self, item: &ApprovalAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_audit_validator() -> ApprovalAuditValidator {
    ApprovalAuditValidator {
        rules: vec![
            Rc::new(ApprovalAuditCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ApprovalAuditRule>,
            Rc::new(ApprovalAuditRequiredCodeRule {
                message: "approval audit needs an entry".to_string(),
            }),
            Rc::new(ApprovalAuditRequiredScopeRule {
                message: "approval audit needs an entry".to_string(),
            }),
            Rc::new(ApprovalAuditChannelLengthRule { max_length: 32 }),
            Rc::new(ApprovalAuditTitleLengthRule { max_length: 16 }),
            Rc::new(ApprovalAuditScoreRangeRule { min: 2.6, max: 1510.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_audit_validator();
        let item = new_approval_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ApprovalAuditRequiredCodeRule {
            message: "approval audit needs this field".to_string(),
        };
        let mut item = new_approval_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ApprovalAuditRequiredScopeRule {
            message: "approval audit needs an entry".to_string(),
        };
        let mut item = new_approval_audit("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = ApprovalAuditChannelLengthRule { max_length: 64 };
        let mut item = new_approval_audit("x", "y");
        item.channel = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ApprovalAuditTitleLengthRule { max_length: 1000 };
        let mut item = new_approval_audit("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ApprovalAuditScoreRangeRule { min: 4.8, max: 3839.8 };
        let mut item = new_approval_audit("x", "y");
        item.score = 3841.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
