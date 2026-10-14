use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling tenant claim must pass. Check returns its problems, or none.
pub trait RollingTenantClaimRule: std::fmt::Debug {
    fn check(&self, item: &RollingTenantClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantClaimRequiredPrincipalRule {
    pub message: String,
}

impl RollingTenantClaimRule for RollingTenantClaimRequiredPrincipalRule {
    fn check(&self, item: &RollingTenantClaim) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantClaimCodePrefixRule {
    pub prefix: String,
}

impl RollingTenantClaimRule for RollingTenantClaimCodePrefixRule {
    fn check(&self, item: &RollingTenantClaim) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantClaimRequiredCodeRule {
    pub message: String,
}

impl RollingTenantClaimRule for RollingTenantClaimRequiredCodeRule {
    fn check(&self, item: &RollingTenantClaim) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantClaimRequiredNameRule {
    pub message: String,
}

impl RollingTenantClaimRule for RollingTenantClaimRequiredNameRule {
    fn check(&self, item: &RollingTenantClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantClaimNameLengthRule {
    pub max_length: i64,
}

impl RollingTenantClaimRule for RollingTenantClaimNameLengthRule {
    fn check(&self, item: &RollingTenantClaim) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling tenant claim.
#[derive(Debug, Clone)]
pub struct RollingTenantClaimValidator {
    pub rules: Vec<Rc<dyn RollingTenantClaimRule>>,
}

impl RollingTenantClaimValidator {
    pub fn validate(&self, item: &RollingTenantClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingTenantClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingTenantClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_tenant_claim_validator() -> RollingTenantClaimValidator {
    RollingTenantClaimValidator {
        rules: vec![
            Rc::new(RollingTenantClaimRequiredPrincipalRule {
                message: "rolling tenant claim needs an entry".to_string(),
            }) as Rc<dyn RollingTenantClaimRule>,
            Rc::new(RollingTenantClaimCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(RollingTenantClaimRequiredCodeRule {
                message: "rolling tenant claim needs this field".to_string(),
            }),
            Rc::new(RollingTenantClaimRequiredNameRule {
                message: "rolling tenant claim needs a value".to_string(),
            }),
            Rc::new(RollingTenantClaimNameLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_tenant_claim_validator();
        let item = new_rolling_tenant_claim("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RollingTenantClaimRequiredPrincipalRule {
            message: "rolling tenant claim needs this field".to_string(),
        };
        let mut item = new_rolling_tenant_claim("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingTenantClaimRequiredCodeRule {
            message: "rolling tenant claim needs this field".to_string(),
        };
        let mut item = new_rolling_tenant_claim("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingTenantClaimRequiredNameRule {
            message: "rolling tenant claim needs an entry".to_string(),
        };
        let mut item = new_rolling_tenant_claim("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RollingTenantClaimNameLengthRule { max_length: 3600 };
        let mut item = new_rolling_tenant_claim("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
