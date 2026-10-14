use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the tenant budget calculation.
pub trait TenantBudgetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceTenantBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantBudgetStep for SourceTenantBudgetAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &TenantBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantBudgetAdjuster {
    pub step: f64,
}

impl TenantBudgetStep for RoundingTenantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantBudget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantBudgetAdjuster {
    pub percent: f64,
}

impl TenantBudgetStep for PercentTenantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantBudgetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantBudgetStep for TieredTenantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantBudget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTenantBudgetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TenantBudgetStep for FlaggedTenantBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TenantBudget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantBudgetEngine {
    pub steps: Vec<Rc<dyn TenantBudgetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantBudgetEngine {
    pub fn compute(&mut self, item: &TenantBudget) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[TenantBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tenant_budget_engine(flags: &flags::Registry) -> TenantBudgetEngine {
    TenantBudgetEngine {
        steps: vec![
            Rc::new(SourceTenantBudgetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.14), ("south".to_string(), 0.8)]),
            }) as Rc<dyn TenantBudgetStep>,
            Rc::new(RoundingTenantBudgetAdjuster { step: 0.5 }),
            Rc::new(PercentTenantBudgetAdjuster { percent: 7.7 }),
            Rc::new(TieredTenantBudgetAdjuster {
                bounds: vec![771.0, 7972.0],
                rates: vec![1.16, 1.0],
            }),
            Rc::new(FlaggedTenantBudgetAdjuster {
                flags: flags.clone(),
                flag: "tenant-budget-primary".to_string(),
                percent: 7.5,
            }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 88139.8,
    }
}
