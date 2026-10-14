use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain draft must pass. Check returns its problems, or none.
pub trait TrustDomainDraftRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDraftRequiredPrincipalRule {
    pub message: String,
}

impl TrustDomainDraftRule for TrustDomainDraftRequiredPrincipalRule {
    fn check(&self, item: &TrustDomainDraft) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDraftCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainDraftRule for TrustDomainDraftCapacityRangeRule {
    fn check(&self, item: &TrustDomainDraft) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainDraftRequiredDescriptionRule {
    pub message: String,
}

impl TrustDomainDraftRule for TrustDomainDraftRequiredDescriptionRule {
    fn check(&self, item: &TrustDomainDraft) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain draft.
#[derive(Debug, Clone)]
pub struct TrustDomainDraftValidator {
    pub rules: Vec<Rc<dyn TrustDomainDraftRule>>,
}

impl TrustDomainDraftValidator {
    pub fn validate(&self, item: &TrustDomainDraft) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_draft_validator() -> TrustDomainDraftValidator {
    TrustDomainDraftValidator {
        rules: vec![
            Rc::new(TrustDomainDraftRequiredPrincipalRule {
                message: "trust domain draft needs this field".to_string(),
            }) as Rc<dyn TrustDomainDraftRule>,
            Rc::new(TrustDomainDraftCapacityRangeRule { min: 32, max: 32 }),
            Rc::new(TrustDomainDraftRequiredDescriptionRule {
                message: "trust domain draft needs an entry".to_string(),
            }),
        ],
    }
}
