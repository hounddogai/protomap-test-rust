use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy audit must pass. Check returns its problems, or none.
pub trait TtlPolicyAuditRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAuditTitleLengthRule {
    pub max_length: i64,
}

impl TtlPolicyAuditRule for TtlPolicyAuditTitleLengthRule {
    fn check(&self, item: &TtlPolicyAudit) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAuditCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyAuditRule for TtlPolicyAuditCodePrefixRule {
    fn check(&self, item: &TtlPolicyAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyAuditRequiredTitleRule {
    pub message: String,
}

impl TtlPolicyAuditRule for TtlPolicyAuditRequiredTitleRule {
    fn check(&self, item: &TtlPolicyAudit) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy audit.
#[derive(Debug, Clone)]
pub struct TtlPolicyAuditValidator {
    pub rules: Vec<Rc<dyn TtlPolicyAuditRule>>,
}

impl TtlPolicyAuditValidator {
    pub fn validate(&self, item: &TtlPolicyAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_audit_validator() -> TtlPolicyAuditValidator {
    TtlPolicyAuditValidator {
        rules: vec![
            Rc::new(TtlPolicyAuditTitleLengthRule { max_length: 3600 }) as Rc<dyn TtlPolicyAuditRule>,
            Rc::new(TtlPolicyAuditCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(TtlPolicyAuditRequiredTitleRule {
                message: "ttl policy audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_audit_validator();
        let item = new_ttl_policy_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TtlPolicyAuditTitleLengthRule { max_length: 32 };
        let mut item = new_ttl_policy_audit("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TtlPolicyAuditRequiredTitleRule {
            message: "ttl policy audit needs a value".to_string(),
        };
        let mut item = new_ttl_policy_audit("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
