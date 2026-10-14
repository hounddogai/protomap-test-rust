use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive region pin ledger must pass. Check returns its problems, or none.
pub trait AdaptiveRegionPinLedgerRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRegionPinLedger) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinLedgerAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl AdaptiveRegionPinLedgerRule for AdaptiveRegionPinLedgerAllowedTenantIdRule {
    fn check(&self, item: &AdaptiveRegionPinLedger) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is unknown", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinLedgerSegmentLengthRule {
    pub max_length: i64,
}

impl AdaptiveRegionPinLedgerRule for AdaptiveRegionPinLedgerSegmentLengthRule {
    fn check(&self, item: &AdaptiveRegionPinLedger) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinLedgerRequiredSegmentRule {
    pub message: String,
}

impl AdaptiveRegionPinLedgerRule for AdaptiveRegionPinLedgerRequiredSegmentRule {
    fn check(&self, item: &AdaptiveRegionPinLedger) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive region pin ledger.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinLedgerValidator {
    pub rules: Vec<Rc<dyn AdaptiveRegionPinLedgerRule>>,
}

impl AdaptiveRegionPinLedgerValidator {
    pub fn validate(&self, item: &AdaptiveRegionPinLedger) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRegionPinLedger) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRegionPinLedger]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_region_pin_ledger_validator() -> AdaptiveRegionPinLedgerValidator {
    AdaptiveRegionPinLedgerValidator {
        rules: vec![
            Rc::new(AdaptiveRegionPinLedgerAllowedTenantIdRule {
                allowed: vec!["tiered".to_string(), "primary".to_string(), "scheduled".to_string()],
            }) as Rc<dyn AdaptiveRegionPinLedgerRule>,
            Rc::new(AdaptiveRegionPinLedgerSegmentLengthRule { max_length: 3600 }),
            Rc::new(AdaptiveRegionPinLedgerRequiredSegmentRule {
                message: "adaptive region pin ledger needs this field".to_string(),
            }),
        ],
    }
}
