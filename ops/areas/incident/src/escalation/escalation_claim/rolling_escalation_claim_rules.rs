use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling escalation claim must pass. Check returns its problems, or none.
pub trait RollingEscalationClaimRule: std::fmt::Debug {
    fn check(&self, item: &RollingEscalationClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaimAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl RollingEscalationClaimRule for RollingEscalationClaimAllowedCategoryRule {
    fn check(&self, item: &RollingEscalationClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is above maximum", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaimIssuerLengthRule {
    pub max_length: i64,
}

impl RollingEscalationClaimRule for RollingEscalationClaimIssuerLengthRule {
    fn check(&self, item: &RollingEscalationClaim) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaimRequiredIssuerRule {
    pub message: String,
}

impl RollingEscalationClaimRule for RollingEscalationClaimRequiredIssuerRule {
    fn check(&self, item: &RollingEscalationClaim) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaimActiveGuardRule {
    pub limit: i64,
}

impl RollingEscalationClaimRule for RollingEscalationClaimActiveGuardRule {
    fn check(&self, item: &RollingEscalationClaim) -> Vec<String> {
        if item.active && (item.ttl_seconds > self.limit) {
            return vec![format!(
                "ttl seconds {} exceeds {} while active",
                item.ttl_seconds, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaimTenantIdLengthRule {
    pub max_length: i64,
}

impl RollingEscalationClaimRule for RollingEscalationClaimTenantIdLengthRule {
    fn check(&self, item: &RollingEscalationClaim) -> Vec<String> {
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
pub struct RollingEscalationClaimRequiredNameRule {
    pub message: String,
}

impl RollingEscalationClaimRule for RollingEscalationClaimRequiredNameRule {
    fn check(&self, item: &RollingEscalationClaim) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling escalation claim.
#[derive(Debug, Clone)]
pub struct RollingEscalationClaimValidator {
    pub rules: Vec<Rc<dyn RollingEscalationClaimRule>>,
}

impl RollingEscalationClaimValidator {
    pub fn validate(&self, item: &RollingEscalationClaim) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingEscalationClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingEscalationClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_escalation_claim_validator() -> RollingEscalationClaimValidator {
    RollingEscalationClaimValidator {
        rules: vec![
            Rc::new(RollingEscalationClaimAllowedCategoryRule {
                allowed: vec!["regional".to_string()],
            }) as Rc<dyn RollingEscalationClaimRule>,
            Rc::new(RollingEscalationClaimIssuerLengthRule { max_length: 255 }),
            Rc::new(RollingEscalationClaimRequiredIssuerRule {
                message: "rolling escalation claim needs a value".to_string(),
            }),
            Rc::new(RollingEscalationClaimActiveGuardRule { limit: 64 }),
            Rc::new(RollingEscalationClaimTenantIdLengthRule { max_length: 255 }),
            Rc::new(RollingEscalationClaimRequiredNameRule {
                message: "rolling escalation claim needs a value".to_string(),
            }),
        ],
    }
}
