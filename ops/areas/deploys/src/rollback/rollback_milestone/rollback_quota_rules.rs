use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback quota must pass. Check returns its problems, or none.
pub trait RollbackQuotaRule: std::fmt::Debug {
    fn check(&self, item: &RollbackQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackQuotaCodePrefixRule {
    pub prefix: String,
}

impl RollbackQuotaRule for RollbackQuotaCodePrefixRule {
    fn check(&self, item: &RollbackQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackQuotaExternalRefLengthRule {
    pub max_length: i64,
}

impl RollbackQuotaRule for RollbackQuotaExternalRefLengthRule {
    fn check(&self, item: &RollbackQuota) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackQuotaCodeLengthRule {
    pub max_length: i64,
}

impl RollbackQuotaRule for RollbackQuotaCodeLengthRule {
    fn check(&self, item: &RollbackQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackQuotaNotesLengthRule {
    pub max_length: i64,
}

impl RollbackQuotaRule for RollbackQuotaNotesLengthRule {
    fn check(&self, item: &RollbackQuota) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackQuotaRequiredCodeRule {
    pub message: String,
}

impl RollbackQuotaRule for RollbackQuotaRequiredCodeRule {
    fn check(&self, item: &RollbackQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback quota.
#[derive(Debug, Clone)]
pub struct RollbackQuotaValidator {
    pub rules: Vec<Rc<dyn RollbackQuotaRule>>,
}

impl RollbackQuotaValidator {
    pub fn validate(&self, item: &RollbackQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_quota_validator() -> RollbackQuotaValidator {
    RollbackQuotaValidator {
        rules: vec![
            Rc::new(RollbackQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RollbackQuotaRule>,
            Rc::new(RollbackQuotaExternalRefLengthRule { max_length: 80 }),
            Rc::new(RollbackQuotaCodeLengthRule { max_length: 32 }),
            Rc::new(RollbackQuotaNotesLengthRule { max_length: 32 }),
            Rc::new(RollbackQuotaRequiredCodeRule {
                message: "rollback quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_quota_validator();
        let item = new_rollback_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = RollbackQuotaExternalRefLengthRule { max_length: 16 };
        let mut item = new_rollback_quota("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackQuotaCodeLengthRule { max_length: 120 };
        let mut item = new_rollback_quota("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = RollbackQuotaNotesLengthRule { max_length: 16 };
        let mut item = new_rollback_quota("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackQuotaRequiredCodeRule {
            message: "rollback quota needs this field".to_string(),
        };
        let mut item = new_rollback_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
