use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress claim must pass. Check returns its problems, or none.
pub trait EgressClaimRule: std::fmt::Debug {
    fn check(&self, item: &EgressClaim) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressClaimDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressClaimRule for EgressClaimDurationMinutesRangeRule {
    fn check(&self, item: &EgressClaim) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressClaimRequiredLabelRule {
    pub message: String,
}

impl EgressClaimRule for EgressClaimRequiredLabelRule {
    fn check(&self, item: &EgressClaim) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressClaimAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl EgressClaimRule for EgressClaimAllowedCodeRule {
    fn check(&self, item: &EgressClaim) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress claim.
#[derive(Debug, Clone)]
pub struct EgressClaimValidator {
    pub rules: Vec<Rc<dyn EgressClaimRule>>,
}

impl EgressClaimValidator {
    pub fn validate(&self, item: &EgressClaim) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressClaim) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressClaim]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_claim_validator() -> EgressClaimValidator {
    EgressClaimValidator {
        rules: vec![
            Rc::new(EgressClaimDurationMinutesRangeRule { min: 32, max: 3600 }) as Rc<dyn EgressClaimRule>,
            Rc::new(EgressClaimRequiredLabelRule {
                message: "egress claim needs a value".to_string(),
            }),
            Rc::new(EgressClaimAllowedCodeRule {
                allowed: vec!["global".to_string()],
            }),
        ],
    }
}
