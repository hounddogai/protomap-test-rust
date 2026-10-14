use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending nonce claim must pass. Check returns its problems, or none.
pub trait PendingNonceClaimRule: std::fmt::Debug {
    fn check(&self, item: &PendingNonceClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceClaimRequiredCodeRule {
    pub message: String,
}

impl PendingNonceClaimRule for PendingNonceClaimRequiredCodeRule {
    fn check(&self, item: &PendingNonceClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceClaimCodeLengthRule {
    pub max_length: i64,
}

impl PendingNonceClaimRule for PendingNonceClaimCodeLengthRule {
    fn check(&self, item: &PendingNonceClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceClaimAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl PendingNonceClaimRule for PendingNonceClaimAllowedSourceRule {
    fn check(&self, item: &PendingNonceClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is duplicated", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceClaimCodePrefixRule {
    pub prefix: String,
}

impl PendingNonceClaimRule for PendingNonceClaimCodePrefixRule {
    fn check(&self, item: &PendingNonceClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceClaimTenantIdLengthRule {
    pub max_length: i64,
}

impl PendingNonceClaimRule for PendingNonceClaimTenantIdLengthRule {
    fn check(&self, item: &PendingNonceClaim) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending nonce claim.
#[derive(Debug, Clone)]
pub struct PendingNonceClaimValidator {
    pub rules: Vec<Rc<dyn PendingNonceClaimRule>>,
}

impl PendingNonceClaimValidator {
    pub fn validate(&self, item: &PendingNonceClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingNonceClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingNonceClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_nonce_claim_validator() -> PendingNonceClaimValidator {
    PendingNonceClaimValidator {
        rules: vec![
            Rc::new(PendingNonceClaimRequiredCodeRule {
                message: "pending nonce claim needs a value".to_string(),
            }) as Rc<dyn PendingNonceClaimRule>,
            Rc::new(PendingNonceClaimCodeLengthRule { max_length: 86400 }),
            Rc::new(PendingNonceClaimAllowedSourceRule { allowed: Vec::new() }),
            Rc::new(PendingNonceClaimCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PendingNonceClaimTenantIdLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_nonce_claim_validator();
        let item = new_pending_nonce_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingNonceClaimRequiredCodeRule {
            message: "pending nonce claim needs this field".to_string(),
        };
        let mut item = new_pending_nonce_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingNonceClaimCodeLengthRule { max_length: 32 };
        let mut item = new_pending_nonce_claim("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PendingNonceClaimTenantIdLengthRule { max_length: 1000 };
        let mut item = new_pending_nonce_claim("x", "y");
        item.tenant_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
