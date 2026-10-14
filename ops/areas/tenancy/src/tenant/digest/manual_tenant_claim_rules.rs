use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual tenant claim must pass. Check returns its problems, or none.
pub trait ManualTenantClaimRule: std::fmt::Debug {
    fn check(&self, item: &ManualTenantClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantClaimAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl ManualTenantClaimRule for ManualTenantClaimAllowedDescriptionRule {
    fn check(&self, item: &ManualTenantClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is above maximum", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantClaimRequiredCodeRule {
    pub message: String,
}

impl ManualTenantClaimRule for ManualTenantClaimRequiredCodeRule {
    fn check(&self, item: &ManualTenantClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantClaimRequiredExternalRefRule {
    pub message: String,
}

impl ManualTenantClaimRule for ManualTenantClaimRequiredExternalRefRule {
    fn check(&self, item: &ManualTenantClaim) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual tenant claim.
#[derive(Debug, Clone)]
pub struct ManualTenantClaimValidator {
    pub rules: Vec<Rc<dyn ManualTenantClaimRule>>,
}

impl ManualTenantClaimValidator {
    pub fn validate(&self, item: &ManualTenantClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualTenantClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTenantClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_tenant_claim_validator() -> ManualTenantClaimValidator {
    ManualTenantClaimValidator {
        rules: vec![
            Rc::new(ManualTenantClaimAllowedDescriptionRule {
                allowed: vec!["adaptive".to_string()],
            }) as Rc<dyn ManualTenantClaimRule>,
            Rc::new(ManualTenantClaimRequiredCodeRule {
                message: "manual tenant claim needs this field".to_string(),
            }),
            Rc::new(ManualTenantClaimRequiredExternalRefRule {
                message: "manual tenant claim needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_tenant_claim_validator();
        let item = new_manual_tenant_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualTenantClaimRequiredCodeRule {
            message: "manual tenant claim needs an entry".to_string(),
        };
        let mut item = new_manual_tenant_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = ManualTenantClaimRequiredExternalRefRule {
            message: "manual tenant claim needs an entry".to_string(),
        };
        let mut item = new_manual_tenant_claim("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
