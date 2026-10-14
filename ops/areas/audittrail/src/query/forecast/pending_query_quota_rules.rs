use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending query quota must pass. Check returns its problems, or none.
pub trait PendingQueryQuotaRule: std::fmt::Debug {
    fn check(&self, item: &PendingQueryQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryQuotaRequiredPrincipalRule {
    pub message: String,
}

impl PendingQueryQuotaRule for PendingQueryQuotaRequiredPrincipalRule {
    fn check(&self, item: &PendingQueryQuota) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryQuotaPrincipalLengthRule {
    pub max_length: i64,
}

impl PendingQueryQuotaRule for PendingQueryQuotaPrincipalLengthRule {
    fn check(&self, item: &PendingQueryQuota) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingQueryQuotaCodeLengthRule {
    pub max_length: i64,
}

impl PendingQueryQuotaRule for PendingQueryQuotaCodeLengthRule {
    fn check(&self, item: &PendingQueryQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending query quota.
#[derive(Debug, Clone)]
pub struct PendingQueryQuotaValidator {
    pub rules: Vec<Rc<dyn PendingQueryQuotaRule>>,
}

impl PendingQueryQuotaValidator {
    pub fn validate(&self, item: &PendingQueryQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingQueryQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingQueryQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_query_quota_validator() -> PendingQueryQuotaValidator {
    PendingQueryQuotaValidator {
        rules: vec![
            Rc::new(PendingQueryQuotaRequiredPrincipalRule {
                message: "pending query quota needs an entry".to_string(),
            }) as Rc<dyn PendingQueryQuotaRule>,
            Rc::new(PendingQueryQuotaPrincipalLengthRule { max_length: 64 }),
            Rc::new(PendingQueryQuotaCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_query_quota_validator();
        let item = new_pending_query_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = PendingQueryQuotaRequiredPrincipalRule {
            message: "pending query quota needs a value".to_string(),
        };
        let mut item = new_pending_query_quota("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = PendingQueryQuotaPrincipalLengthRule { max_length: 86400 };
        let mut item = new_pending_query_quota("x", "y");
        item.principal = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingQueryQuotaCodeLengthRule { max_length: 120 };
        let mut item = new_pending_query_quota("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
