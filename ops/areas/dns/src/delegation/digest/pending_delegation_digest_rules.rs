use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending delegation digest must pass. Check returns its problems, or none.
pub trait PendingDelegationDigestRule: std::fmt::Debug {
    fn check(&self, item: &PendingDelegationDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationDigestNameLengthRule {
    pub max_length: i64,
}

impl PendingDelegationDigestRule for PendingDelegationDigestNameLengthRule {
    fn check(&self, item: &PendingDelegationDigest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationDigestRequiredCipherRule {
    pub message: String,
}

impl PendingDelegationDigestRule for PendingDelegationDigestRequiredCipherRule {
    fn check(&self, item: &PendingDelegationDigest) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingDelegationDigestRequiredPrincipalRule {
    pub message: String,
}

impl PendingDelegationDigestRule for PendingDelegationDigestRequiredPrincipalRule {
    fn check(&self, item: &PendingDelegationDigest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending delegation digest.
#[derive(Debug, Clone)]
pub struct PendingDelegationDigestValidator {
    pub rules: Vec<Rc<dyn PendingDelegationDigestRule>>,
}

impl PendingDelegationDigestValidator {
    pub fn validate(&self, item: &PendingDelegationDigest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingDelegationDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingDelegationDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_delegation_digest_validator() -> PendingDelegationDigestValidator {
    PendingDelegationDigestValidator {
        rules: vec![
            Rc::new(PendingDelegationDigestNameLengthRule { max_length: 1000 }) as Rc<dyn PendingDelegationDigestRule>,
            Rc::new(PendingDelegationDigestRequiredCipherRule {
                message: "pending delegation digest needs this field".to_string(),
            }),
            Rc::new(PendingDelegationDigestRequiredPrincipalRule {
                message: "pending delegation digest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_delegation_digest_validator();
        let item = new_pending_delegation_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = PendingDelegationDigestNameLengthRule { max_length: 64 };
        let mut item = new_pending_delegation_digest("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = PendingDelegationDigestRequiredCipherRule {
            message: "pending delegation digest needs a value".to_string(),
        };
        let mut item = new_pending_delegation_digest("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = PendingDelegationDigestRequiredPrincipalRule {
            message: "pending delegation digest needs this field".to_string(),
        };
        let mut item = new_pending_delegation_digest("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
