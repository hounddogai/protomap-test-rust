use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual tenant rule calculation.
pub trait ManualTenantRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTenantRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualTenantRuleAdjuster {
    pub cap: f64,
}

impl ManualTenantRuleModifier for CapManualTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualTenantRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualTenantRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualTenantRuleModifier for TieredManualTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualTenantRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameManualTenantRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ManualTenantRuleModifier for NameManualTenantRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ManualTenantRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantRuleQuantityOffsetAdjuster {
    pub weight: f64,
}

impl ManualTenantRuleModifier for ManualTenantRuleQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &ManualTenantRule, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualTenantRuleAdjuster {
    pub percent: f64,
}

impl ManualTenantRuleModifier for PercentManualTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualTenantRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualTenantRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualTenantRuleModifier for FlaggedManualTenantRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualTenantRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual tenant rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTenantRuleEngine {
    pub steps: Vec<Rc<dyn ManualTenantRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTenantRuleEngine {
    pub fn price(&mut self, item: &ManualTenantRule) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualTenantRuleModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[ManualTenantRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_tenant_rule_engine(flags: &flags::Registry) -> ManualTenantRuleEngine {
    ManualTenantRuleEngine {
        steps: vec![
            Rc::new(CapManualTenantRuleAdjuster { cap: 22107.0 }) as Rc<dyn ManualTenantRuleModifier>,
            Rc::new(TieredManualTenantRuleAdjuster {
                bounds: vec![636.0, 1559.0, 6833.0],
                rates: vec![0.86, 0.91, 1.14],
            }),
            Rc::new(NameManualTenantRuleAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.15)]),
            }),
            Rc::new(ManualTenantRuleQuantityOffsetAdjuster { weight: 0.25 }),
            Rc::new(PercentManualTenantRuleAdjuster { percent: 17.4 }),
            Rc::new(FlaggedManualTenantRuleAdjuster {
                flags: flags.clone(),
                flag: "manual-tenant-rule-adaptive".to_string(),
                percent: -6.1,
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 72014.7,
    }
}
