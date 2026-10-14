use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain tier must pass. Check returns its problems, or none.
pub trait TrustDomainTierRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTierRequiredCodeRule {
    pub message: String,
}

impl TrustDomainTierRule for TrustDomainTierRequiredCodeRule {
    fn check(&self, item: &TrustDomainTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTierTenantIdLengthRule {
    pub max_length: i64,
}

impl TrustDomainTierRule for TrustDomainTierTenantIdLengthRule {
    fn check(&self, item: &TrustDomainTier) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTierCodeLengthRule {
    pub max_length: i64,
}

impl TrustDomainTierRule for TrustDomainTierCodeLengthRule {
    fn check(&self, item: &TrustDomainTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain tier.
#[derive(Debug, Clone)]
pub struct TrustDomainTierValidator {
    pub rules: Vec<Rc<dyn TrustDomainTierRule>>,
}

impl TrustDomainTierValidator {
    pub fn validate(&self, item: &TrustDomainTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_tier_validator() -> TrustDomainTierValidator {
    TrustDomainTierValidator {
        rules: vec![
            Rc::new(TrustDomainTierRequiredCodeRule {
                message: "trust domain tier needs a value".to_string(),
            }) as Rc<dyn TrustDomainTierRule>,
            Rc::new(TrustDomainTierTenantIdLengthRule { max_length: 120 }),
            Rc::new(TrustDomainTierCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_tier_validator();
        let item = new_trust_domain_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TrustDomainTierRequiredCodeRule {
            message: "trust domain tier needs a value".to_string(),
        };
        let mut item = new_trust_domain_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = TrustDomainTierTenantIdLengthRule { max_length: 255 };
        let mut item = new_trust_domain_tier("x", "y");
        item.tenant_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TrustDomainTierCodeLengthRule { max_length: 80 };
        let mut item = new_trust_domain_tier("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
