use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant checkpoint calculation.
pub trait TenantCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantCheckpointSequenceOffsetAdjuster {
    pub weight: f64,
}

impl TenantCheckpointAdjuster for TenantCheckpointSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &TenantCheckpoint, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantCheckpointAdjuster for TieredTenantCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantCheckpointAdjuster {
    pub step: f64,
}

impl TenantCheckpointAdjuster for RoundingTenantCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantCheckpointAdjuster {
    pub minimum: f64,
}

impl TenantCheckpointAdjuster for MinimumTenantCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantCheckpointAdjuster {
    pub cap: f64,
}

impl TenantCheckpointAdjuster for CapTenantCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTenantCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantCheckpointAdjuster for CategoryTenantCheckpointAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &TenantCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tenant checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantCheckpointEngine {
    pub steps: Vec<Rc<dyn TenantCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantCheckpointEngine {
    pub fn evaluate(&mut self, item: &TenantCheckpoint) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[TenantCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tenant_checkpoint_engine() -> TenantCheckpointEngine {
    TenantCheckpointEngine {
        steps: vec![
            Rc::new(TenantCheckpointSequenceOffsetAdjuster { weight: 1.78 }) as Rc<dyn TenantCheckpointAdjuster>,
            Rc::new(TieredTenantCheckpointAdjuster {
                bounds: vec![345.0, 673.0],
                rates: vec![1.07, 1.15],
            }),
            Rc::new(RoundingTenantCheckpointAdjuster { step: 0.25 }),
            Rc::new(MinimumTenantCheckpointAdjuster { minimum: 39.2 }),
            Rc::new(CapTenantCheckpointAdjuster { cap: 26074.0 }),
            Rc::new(CategoryTenantCheckpointAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.26), ("central".to_string(), 1.04)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 30933.5,
    }
}
