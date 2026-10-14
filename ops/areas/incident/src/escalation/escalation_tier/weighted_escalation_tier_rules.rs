use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted escalation tier must pass. Check returns its problems, or none.
pub trait WeightedEscalationTierRule: std::fmt::Debug {
    fn check(&self, item: &WeightedEscalationTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierRequiredNotesRule {
    pub message: String,
}

impl WeightedEscalationTierRule for WeightedEscalationTierRequiredNotesRule {
    fn check(&self, item: &WeightedEscalationTier) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedEscalationTierRule for WeightedEscalationTierQuantityRangeRule {
    fn check(&self, item: &WeightedEscalationTier) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierRequiredRegionRule {
    pub message: String,
}

impl WeightedEscalationTierRule for WeightedEscalationTierRequiredRegionRule {
    fn check(&self, item: &WeightedEscalationTier) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierRequiredCodeRule {
    pub message: String,
}

impl WeightedEscalationTierRule for WeightedEscalationTierRequiredCodeRule {
    fn check(&self, item: &WeightedEscalationTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedEscalationTierRule for WeightedEscalationTierCostRangeRule {
    fn check(&self, item: &WeightedEscalationTier) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedEscalationTierRequiredTitleRule {
    pub message: String,
}

impl WeightedEscalationTierRule for WeightedEscalationTierRequiredTitleRule {
    fn check(&self, item: &WeightedEscalationTier) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted escalation tier.
#[derive(Debug, Clone)]
pub struct WeightedEscalationTierValidator {
    pub rules: Vec<Rc<dyn WeightedEscalationTierRule>>,
}

impl WeightedEscalationTierValidator {
    pub fn validate(&self, item: &WeightedEscalationTier) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedEscalationTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedEscalationTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_escalation_tier_validator() -> WeightedEscalationTierValidator {
    WeightedEscalationTierValidator {
        rules: vec![
            Rc::new(WeightedEscalationTierRequiredNotesRule {
                message: "weighted escalation tier needs a value".to_string(),
            }) as Rc<dyn WeightedEscalationTierRule>,
            Rc::new(WeightedEscalationTierQuantityRangeRule { min: 80, max: 120 }),
            Rc::new(WeightedEscalationTierRequiredRegionRule {
                message: "weighted escalation tier needs this field".to_string(),
            }),
            Rc::new(WeightedEscalationTierRequiredCodeRule {
                message: "weighted escalation tier needs a value".to_string(),
            }),
            Rc::new(WeightedEscalationTierCostRangeRule { min: 0.4, max: 4669.9 }),
            Rc::new(WeightedEscalationTierRequiredTitleRule {
                message: "weighted escalation tier needs a value".to_string(),
            }),
        ],
    }
}
