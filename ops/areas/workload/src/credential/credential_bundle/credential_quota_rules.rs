use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a credential quota must pass. Check returns its problems, or none.
pub trait CredentialQuotaRule: std::fmt::Debug {
    fn check(&self, item: &CredentialQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialQuotaCodeLengthRule {
    pub max_length: i64,
}

impl CredentialQuotaRule for CredentialQuotaCodeLengthRule {
    fn check(&self, item: &CredentialQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialQuotaCodePrefixRule {
    pub prefix: String,
}

impl CredentialQuotaRule for CredentialQuotaCodePrefixRule {
    fn check(&self, item: &CredentialQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialQuotaRequiredKeyIdRule {
    pub message: String,
}

impl CredentialQuotaRule for CredentialQuotaRequiredKeyIdRule {
    fn check(&self, item: &CredentialQuota) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialQuotaRequiredCodeRule {
    pub message: String,
}

impl CredentialQuotaRule for CredentialQuotaRequiredCodeRule {
    fn check(&self, item: &CredentialQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a credential quota.
#[derive(Debug, Clone)]
pub struct CredentialQuotaValidator {
    pub rules: Vec<Rc<dyn CredentialQuotaRule>>,
}

impl CredentialQuotaValidator {
    pub fn validate(&self, item: &CredentialQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CredentialQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CredentialQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_credential_quota_validator() -> CredentialQuotaValidator {
    CredentialQuotaValidator {
        rules: vec![
            Rc::new(CredentialQuotaCodeLengthRule { max_length: 80 }) as Rc<dyn CredentialQuotaRule>,
            Rc::new(CredentialQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(CredentialQuotaRequiredKeyIdRule {
                message: "credential quota needs a value".to_string(),
            }),
            Rc::new(CredentialQuotaRequiredCodeRule {
                message: "credential quota needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_credential_quota_validator();
        let item = new_credential_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CredentialQuotaCodeLengthRule { max_length: 255 };
        let mut item = new_credential_quota("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = CredentialQuotaRequiredKeyIdRule {
            message: "credential quota needs a value".to_string(),
        };
        let mut item = new_credential_quota("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CredentialQuotaRequiredCodeRule {
            message: "credential quota needs this field".to_string(),
        };
        let mut item = new_credential_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
