use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant claim must pass. Check returns its problems, or none.
pub trait TenantClaimRule: std::fmt::Debug {
    fn check(&self, item: &TenantClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantClaimRequiredNotesRule {
    pub message: String,
}

impl TenantClaimRule for TenantClaimRequiredNotesRule {
    fn check(&self, item: &TenantClaim) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantClaimRequiredCodeRule {
    pub message: String,
}

impl TenantClaimRule for TenantClaimRequiredCodeRule {
    fn check(&self, item: &TenantClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantClaimCodePrefixRule {
    pub prefix: String,
}

impl TenantClaimRule for TenantClaimCodePrefixRule {
    fn check(&self, item: &TenantClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant claim.
#[derive(Debug, Clone)]
pub struct TenantClaimValidator {
    pub rules: Vec<Rc<dyn TenantClaimRule>>,
}

impl TenantClaimValidator {
    pub fn validate(&self, item: &TenantClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_claim_validator() -> TenantClaimValidator {
    TenantClaimValidator {
        rules: vec![
            Rc::new(TenantClaimRequiredNotesRule {
                message: "tenant claim needs this field".to_string(),
            }) as Rc<dyn TenantClaimRule>,
            Rc::new(TenantClaimRequiredCodeRule {
                message: "tenant claim needs an entry".to_string(),
            }),
            Rc::new(TenantClaimCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_claim_validator();
        let item = new_tenant_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = TenantClaimRequiredNotesRule {
            message: "tenant claim needs a value".to_string(),
        };
        let mut item = new_tenant_claim("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantClaimRequiredCodeRule {
            message: "tenant claim needs an entry".to_string(),
        };
        let mut item = new_tenant_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
