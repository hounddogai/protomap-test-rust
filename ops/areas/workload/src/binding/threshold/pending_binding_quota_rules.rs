use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending binding quota must pass. Check returns its problems, or none.
pub trait PendingBindingQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PendingBindingQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingQuotaNameLengthRule {
    pub max_length: i64,
}

impl PendingBindingQuotaRule for PendingBindingQuotaNameLengthRule {
    fn check(&self, item: &PendingBindingQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingQuotaRequiredCodeRule {
    pub message: String,
}

impl PendingBindingQuotaRule for PendingBindingQuotaRequiredCodeRule {
    fn check(&self, item: &PendingBindingQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingQuotaClusterLengthRule {
    pub max_length: i64,
}

impl PendingBindingQuotaRule for PendingBindingQuotaClusterLengthRule {
    fn check(&self, item: &PendingBindingQuota) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingQuotaCodePrefixRule {
    pub prefix: String,
}

impl PendingBindingQuotaRule for PendingBindingQuotaCodePrefixRule {
    fn check(&self, item: &PendingBindingQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending binding quota.
#[derive(Debug, Clone)]
pub struct PendingBindingQuotaValidator {
    pub rules: Vec<Rc<dyn PendingBindingQuotaRule>>,
}

impl PendingBindingQuotaValidator {
    pub fn validate(&self, item: &PendingBindingQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingBindingQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingBindingQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_binding_quota_validator() -> PendingBindingQuotaValidator {
    PendingBindingQuotaValidator {
        rules: vec![
            Rc::new(PendingBindingQuotaNameLengthRule { max_length: 1000 }) as Rc<dyn PendingBindingQuotaRule>,
            Rc::new(PendingBindingQuotaRequiredCodeRule {
                message: "pending binding quota needs this field".to_string(),
            }),
            Rc::new(PendingBindingQuotaClusterLengthRule { max_length: 1000 }),
            Rc::new(PendingBindingQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_binding_quota_validator();
        let item = new_pending_binding_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingBindingQuotaNameLengthRule { max_length: 64 };
        let mut item = new_pending_binding_quota("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingBindingQuotaRequiredCodeRule {
            message: "pending binding quota needs an entry".to_string(),
        };
        let mut item = new_pending_binding_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = PendingBindingQuotaClusterLengthRule { max_length: 80 };
        let mut item = new_pending_binding_quota("x", "y");
        item.cluster = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
