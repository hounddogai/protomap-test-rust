use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant proposal must pass. Check returns its problems, or none.
pub trait TenantProposalRule: std::fmt::Debug {
    fn check(&self, item: &TenantProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposalAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl TenantProposalRule for TenantProposalAllowedUnitRule {
    fn check(&self, item: &TenantProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is duplicated", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposalLabelLengthRule {
    pub max_length: i64,
}

impl TenantProposalRule for TenantProposalLabelLengthRule {
    fn check(&self, item: &TenantProposal) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposalDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantProposalRule for TenantProposalDiscountRangeRule {
    fn check(&self, item: &TenantProposal) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposalRequiredExternalRefRule {
    pub message: String,
}

impl TenantProposalRule for TenantProposalRequiredExternalRefRule {
    fn check(&self, item: &TenantProposal) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposalEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantProposalRule for TenantProposalEndsAtRangeRule {
    fn check(&self, item: &TenantProposal) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProposalRequiredUnitRule {
    pub message: String,
}

impl TenantProposalRule for TenantProposalRequiredUnitRule {
    fn check(&self, item: &TenantProposal) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant proposal.
#[derive(Debug, Clone)]
pub struct TenantProposalValidator {
    pub rules: Vec<Rc<dyn TenantProposalRule>>,
}

impl TenantProposalValidator {
    pub fn validate(&self, item: &TenantProposal) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_proposal_validator() -> TenantProposalValidator {
    TenantProposalValidator {
        rules: vec![
            Rc::new(TenantProposalAllowedUnitRule {
                allowed: vec!["batched".to_string()],
            }) as Rc<dyn TenantProposalRule>,
            Rc::new(TenantProposalLabelLengthRule { max_length: 3600 }),
            Rc::new(TenantProposalDiscountRangeRule { min: 2.7, max: 1328.5 }),
            Rc::new(TenantProposalRequiredExternalRefRule {
                message: "tenant proposal needs a value".to_string(),
            }),
            Rc::new(TenantProposalEndsAtRangeRule { min: 64, max: 255 }),
            Rc::new(TenantProposalRequiredUnitRule {
                message: "tenant proposal needs this field".to_string(),
            }),
        ],
    }
}
