use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict dashboard slot must pass. Check returns its problems, or none.
pub trait StrictDashboardSlotRule: std::fmt::Debug {
    fn check(&self, item: &StrictDashboardSlot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardSlotRequiredNamespaceRule {
    pub message: String,
}

impl StrictDashboardSlotRule for StrictDashboardSlotRequiredNamespaceRule {
    fn check(&self, item: &StrictDashboardSlot) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardSlotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictDashboardSlotRule for StrictDashboardSlotRevisionRangeRule {
    fn check(&self, item: &StrictDashboardSlot) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardSlotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictDashboardSlotRule for StrictDashboardSlotQuantityRangeRule {
    fn check(&self, item: &StrictDashboardSlot) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictDashboardSlotCurrencyLengthRule {
    pub max_length: i64,
}

impl StrictDashboardSlotRule for StrictDashboardSlotCurrencyLengthRule {
    fn check(&self, item: &StrictDashboardSlot) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict dashboard slot.
#[derive(Debug, Clone)]
pub struct StrictDashboardSlotValidator {
    pub rules: Vec<Rc<dyn StrictDashboardSlotRule>>,
}

impl StrictDashboardSlotValidator {
    pub fn validate(&self, item: &StrictDashboardSlot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictDashboardSlot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictDashboardSlot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_dashboard_slot_validator() -> StrictDashboardSlotValidator {
    StrictDashboardSlotValidator {
        rules: vec![
            Rc::new(StrictDashboardSlotRequiredNamespaceRule {
                message: "strict dashboard slot needs a value".to_string(),
            }) as Rc<dyn StrictDashboardSlotRule>,
            Rc::new(StrictDashboardSlotRevisionRangeRule { min: 16, max: 1000 }),
            Rc::new(StrictDashboardSlotQuantityRangeRule { min: 3600, max: 86400 }),
            Rc::new(StrictDashboardSlotCurrencyLengthRule { max_length: 1000 }),
        ],
    }
}
