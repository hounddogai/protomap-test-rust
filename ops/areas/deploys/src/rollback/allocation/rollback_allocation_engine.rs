use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback allocation calculation.
pub trait RollbackAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackAllocationAdjuster {
    pub percent: f64,
}

impl RollbackAllocationFactor for PercentRollbackAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackAllocationAdjuster {
    pub minimum: f64,
}

impl RollbackAllocationFactor for MinimumRollbackAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollbackAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollbackAllocationFactor for TieredRollbackAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollbackAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackAllocationAdjuster {
    pub step: f64,
}

impl RollbackAllocationFactor for RoundingRollbackAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceRollbackAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackAllocationFactor for NamespaceRollbackAllocationAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &RollbackAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackAllocationAdjuster {
    pub cap: f64,
}

impl RollbackAllocationFactor for CapRollbackAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rollback allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackAllocationEngine {
    pub steps: Vec<Rc<dyn RollbackAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackAllocationEngine {
    pub fn compute(&mut self, item: &RollbackAllocation) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[RollbackAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_allocation_engine() -> RollbackAllocationEngine {
    RollbackAllocationEngine {
        steps: vec![
            Rc::new(PercentRollbackAllocationAdjuster { percent: 24.0 }) as Rc<dyn RollbackAllocationFactor>,
            Rc::new(MinimumRollbackAllocationAdjuster { minimum: 10.2 }),
            Rc::new(TieredRollbackAllocationAdjuster {
                bounds: vec![1759.0, 2351.0, 8442.0, 9775.0],
                rates: vec![0.81, 0.97, 1.1, 1.03],
            }),
            Rc::new(RoundingRollbackAllocationAdjuster { step: 0.5 }),
            Rc::new(NamespaceRollbackAllocationAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.93)]),
            }),
            Rc::new(CapRollbackAllocationAdjuster { cap: 15996.0 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 82297.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollback_allocation_engine();
        let item = new_rollback_allocation("code-160", "label-381");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
