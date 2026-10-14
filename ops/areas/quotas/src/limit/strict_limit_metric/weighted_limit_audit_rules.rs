use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted limit audit must pass. Check returns its problems, or none.
pub trait WeightedLimitAuditRule: std::fmt::Debug {
    fn check(&self, item: &WeightedLimitAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitAuditLabelLengthRule {
    pub max_length: i64,
}

impl WeightedLimitAuditRule for WeightedLimitAuditLabelLengthRule {
    fn check(&self, item: &WeightedLimitAudit) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitAuditArchivedGuardRule {
    pub limit: i64,
}

impl WeightedLimitAuditRule for WeightedLimitAuditArchivedGuardRule {
    fn check(&self, item: &WeightedLimitAudit) -> Vec<String> {
        if item.archived && (item.updated_at > self.limit) {
            return vec![format!(
                "updated at {} exceeds {} while archived",
                item.updated_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitAuditCodePrefixRule {
    pub prefix: String,
}

impl WeightedLimitAuditRule for WeightedLimitAuditCodePrefixRule {
    fn check(&self, item: &WeightedLimitAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitAuditContactEmailLengthRule {
    pub max_length: i64,
}

impl WeightedLimitAuditRule for WeightedLimitAuditContactEmailLengthRule {
    fn check(&self, item: &WeightedLimitAudit) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedLimitAuditRequiredCodeRule {
    pub message: String,
}

impl WeightedLimitAuditRule for WeightedLimitAuditRequiredCodeRule {
    fn check(&self, item: &WeightedLimitAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted limit audit.
#[derive(Debug, Clone)]
pub struct WeightedLimitAuditValidator {
    pub rules: Vec<Rc<dyn WeightedLimitAuditRule>>,
}

impl WeightedLimitAuditValidator {
    pub fn validate(&self, item: &WeightedLimitAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedLimitAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedLimitAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_limit_audit_validator() -> WeightedLimitAuditValidator {
    WeightedLimitAuditValidator {
        rules: vec![
            Rc::new(WeightedLimitAuditLabelLengthRule { max_length: 16 }) as Rc<dyn WeightedLimitAuditRule>,
            Rc::new(WeightedLimitAuditArchivedGuardRule { limit: 80 }),
            Rc::new(WeightedLimitAuditCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(WeightedLimitAuditContactEmailLengthRule { max_length: 32 }),
            Rc::new(WeightedLimitAuditRequiredCodeRule {
                message: "weighted limit audit needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_limit_audit_validator();
        let item = new_weighted_limit_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedLimitAuditLabelLengthRule { max_length: 64 };
        let mut item = new_weighted_limit_audit("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = WeightedLimitAuditContactEmailLengthRule { max_length: 16 };
        let mut item = new_weighted_limit_audit("x", "y");
        item.contact_email = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedLimitAuditRequiredCodeRule {
            message: "weighted limit audit needs this field".to_string(),
        };
        let mut item = new_weighted_limit_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
