use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the migration rule calculation.
pub trait MigrationRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MigrationRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMigrationRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MigrationRuleFactor for TieredMigrationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MigrationRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMigrationRuleAdjuster {
    pub minimum: f64,
}

impl MigrationRuleFactor for MinimumMigrationRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MigrationRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMigrationRuleAdjuster {
    pub cap: f64,
}

impl MigrationRuleFactor for CapMigrationRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MigrationRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedMigrationRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MigrationRuleFactor for FlaggedMigrationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MigrationRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMigrationRuleAdjuster {
    pub step: f64,
}

impl MigrationRuleFactor for RoundingMigrationRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MigrationRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the migration rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MigrationRuleEngine {
    pub steps: Vec<Rc<dyn MigrationRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MigrationRuleEngine {
    pub fn evaluate(&mut self, item: &MigrationRule) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[MigrationRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_migration_rule_engine(flags: &flags::Registry) -> MigrationRuleEngine {
    MigrationRuleEngine {
        steps: vec![
            Rc::new(TieredMigrationRuleAdjuster {
                bounds: vec![611.0, 3824.0],
                rates: vec![0.87, 0.97],
            }) as Rc<dyn MigrationRuleFactor>,
            Rc::new(MinimumMigrationRuleAdjuster { minimum: 34.6 }),
            Rc::new(CapMigrationRuleAdjuster { cap: 12214.0 }),
            Rc::new(FlaggedMigrationRuleAdjuster {
                flags: flags.clone(),
                flag: "migration-rule-manual".to_string(),
                percent: 2.5,
            }),
            Rc::new(RoundingMigrationRuleAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 7364.6,
    }
}
