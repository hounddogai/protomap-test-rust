use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending trust domain slot must pass. Check returns its problems, or none.
pub trait PendingTrustDomainSlotRule: std::fmt::Debug {
    fn check(&self, item: &PendingTrustDomainSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainSlotLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingTrustDomainSlotRule for PendingTrustDomainSlotLimitRangeRule {
    fn check(&self, item: &PendingTrustDomainSlot) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainSlotAllowedSourceRule {
    pub allowed: Vec<String>,
}

impl PendingTrustDomainSlotRule for PendingTrustDomainSlotAllowedSourceRule {
    fn check(&self, item: &PendingTrustDomainSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.source))) {
            return vec![format!("source {} is blocked", item.source)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainSlotRequiredCodeRule {
    pub message: String,
}

impl PendingTrustDomainSlotRule for PendingTrustDomainSlotRequiredCodeRule {
    fn check(&self, item: &PendingTrustDomainSlot) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending trust domain slot.
#[derive(Debug, Clone)]
pub struct PendingTrustDomainSlotValidator {
    pub rules: Vec<Rc<dyn PendingTrustDomainSlotRule>>,
}

impl PendingTrustDomainSlotValidator {
    pub fn validate(&self, item: &PendingTrustDomainSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingTrustDomainSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTrustDomainSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_trust_domain_slot_validator() -> PendingTrustDomainSlotValidator {
    PendingTrustDomainSlotValidator {
        rules: vec![
            Rc::new(PendingTrustDomainSlotLimitRangeRule { min: 16, max: 80 }) as Rc<dyn PendingTrustDomainSlotRule>,
            Rc::new(PendingTrustDomainSlotAllowedSourceRule {
                allowed: vec!["manual".to_string(), "seasonal".to_string(), "seasonal".to_string()],
            }),
            Rc::new(PendingTrustDomainSlotRequiredCodeRule {
                message: "pending trust domain slot needs an entry".to_string(),
            }),
        ],
    }
}
