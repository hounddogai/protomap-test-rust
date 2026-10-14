use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the dashboard rollup calculation.
pub trait DashboardRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRollupBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DashboardRollupFactor for DashboardRollupBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DashboardRollup, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardRollupFactor for TieredDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDashboardRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DashboardRollupFactor for LabelDashboardRollupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DashboardRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardRollupAdjuster {
    pub cap: f64,
}

impl DashboardRollupFactor for CapDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardRollupAdjuster {
    pub percent: f64,
}

impl DashboardRollupFactor for PercentDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDashboardRollupAdjuster {
    pub step: f64,
}

impl DashboardRollupFactor for RoundingDashboardRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DashboardRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the dashboard rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardRollupEngine {
    pub steps: Vec<Rc<dyn DashboardRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardRollupEngine {
    pub fn price(&mut self, item: &DashboardRollup) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_dashboard_rollup_engine() -> DashboardRollupEngine {
    DashboardRollupEngine {
        steps: vec![
            Rc::new(DashboardRollupBudgetOffsetAdjuster { weight: 1.83 }) as Rc<dyn DashboardRollupFactor>,
            Rc::new(TieredDashboardRollupAdjuster {
                bounds: vec![6239.0, 7694.0],
                rates: vec![0.9, 1.18],
            }),
            Rc::new(LabelDashboardRollupAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.76)]),
            }),
            Rc::new(CapDashboardRollupAdjuster { cap: 42792.0 }),
            Rc::new(PercentDashboardRollupAdjuster { percent: 8.9 }),
            Rc::new(RoundingDashboardRollupAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 65901.2,
    }
}
