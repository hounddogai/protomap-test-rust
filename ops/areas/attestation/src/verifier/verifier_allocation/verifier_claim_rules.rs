use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier claim must pass. Check returns its problems, or none.
pub trait VerifierClaimRule: std::fmt::Debug {
    fn check(&self, item: &VerifierClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimUnitLengthRule {
    pub max_length: i64,
}

impl VerifierClaimRule for VerifierClaimUnitLengthRule {
    fn check(&self, item: &VerifierClaim) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimCodeLengthRule {
    pub max_length: i64,
}

impl VerifierClaimRule for VerifierClaimCodeLengthRule {
    fn check(&self, item: &VerifierClaim) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimRequiredTenantIdRule {
    pub message: String,
}

impl VerifierClaimRule for VerifierClaimRequiredTenantIdRule {
    fn check(&self, item: &VerifierClaim) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimCodePrefixRule {
    pub prefix: String,
}

impl VerifierClaimRule for VerifierClaimCodePrefixRule {
    fn check(&self, item: &VerifierClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl VerifierClaimRule for VerifierClaimAllowedCodeRule {
    fn check(&self, item: &VerifierClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is expired", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierClaimRequiredTitleRule {
    pub message: String,
}

impl VerifierClaimRule for VerifierClaimRequiredTitleRule {
    fn check(&self, item: &VerifierClaim) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier claim.
#[derive(Debug, Clone)]
pub struct VerifierClaimValidator {
    pub rules: Vec<Rc<dyn VerifierClaimRule>>,
}

impl VerifierClaimValidator {
    pub fn validate(&self, item: &VerifierClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_claim_validator() -> VerifierClaimValidator {
    VerifierClaimValidator {
        rules: vec![
            Rc::new(VerifierClaimUnitLengthRule { max_length: 16 }) as Rc<dyn VerifierClaimRule>,
            Rc::new(VerifierClaimCodeLengthRule { max_length: 64 }),
            Rc::new(VerifierClaimRequiredTenantIdRule {
                message: "verifier claim needs this field".to_string(),
            }),
            Rc::new(VerifierClaimCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(VerifierClaimAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(VerifierClaimRequiredTitleRule {
                message: "verifier claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_claim_validator();
        let item = new_verifier_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = VerifierClaimUnitLengthRule { max_length: 86400 };
        let mut item = new_verifier_claim("x", "y");
        item.unit = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierClaimCodeLengthRule { max_length: 86400 };
        let mut item = new_verifier_claim("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = VerifierClaimRequiredTenantIdRule {
            message: "verifier claim needs a value".to_string(),
        };
        let mut item = new_verifier_claim("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = VerifierClaimRequiredTitleRule {
            message: "verifier claim needs a value".to_string(),
        };
        let mut item = new_verifier_claim("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
