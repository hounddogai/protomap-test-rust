use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling credential slot must pass. Check returns its problems, or none.
pub trait RollingCredentialSlotRule: std::fmt::Debug {
    fn check(&self, item: &RollingCredentialSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSlotAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollingCredentialSlotRule for RollingCredentialSlotAllowedNameRule {
    fn check(&self, item: &RollingCredentialSlot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSlotUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingCredentialSlotRule for RollingCredentialSlotUpdatedAtRangeRule {
    fn check(&self, item: &RollingCredentialSlot) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingCredentialSlotRequiredKeyIdRule {
    pub message: String,
}

impl RollingCredentialSlotRule for RollingCredentialSlotRequiredKeyIdRule {
    fn check(&self, item: &RollingCredentialSlot) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling credential slot.
#[derive(Debug, Clone)]
pub struct RollingCredentialSlotValidator {
    pub rules: Vec<Rc<dyn RollingCredentialSlotRule>>,
}

impl RollingCredentialSlotValidator {
    pub fn validate(&self, item: &RollingCredentialSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingCredentialSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingCredentialSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_credential_slot_validator() -> RollingCredentialSlotValidator {
    RollingCredentialSlotValidator {
        rules: vec![
            Rc::new(RollingCredentialSlotAllowedNameRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn RollingCredentialSlotRule>,
            Rc::new(RollingCredentialSlotUpdatedAtRangeRule { min: 64, max: 86400 }),
            Rc::new(RollingCredentialSlotRequiredKeyIdRule {
                message: "rolling credential slot needs this field".to_string(),
            }),
        ],
    }
}
