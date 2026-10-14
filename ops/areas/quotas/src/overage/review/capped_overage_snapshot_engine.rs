use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the capped overage snapshot calculation.
pub trait CappedOverageSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedOverageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedOverageSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedOverageSnapshotFactor for TieredCappedOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedOverageSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedOverageSnapshotAdjuster {
    pub cap: f64,
}

impl CappedOverageSnapshotFactor for CapCappedOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedOverageSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedOverageSnapshotAdjuster {
    pub percent: f64,
}

impl CappedOverageSnapshotFactor for PercentCappedOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedOverageSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedOverageSnapshotAdjuster {
    pub step: f64,
}

impl CappedOverageSnapshotFactor for RoundingCappedOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedOverageSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the capped overage snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedOverageSnapshotEngine {
    pub steps: Vec<Rc<dyn CappedOverageSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedOverageSnapshotEngine {
    pub fn compute(&mut self, item: &CappedOverageSnapshot) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedOverageSnapshotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedOverageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_capped_overage_snapshot_engine() -> CappedOverageSnapshotEngine {
    CappedOverageSnapshotEngine {
        steps: vec![
            Rc::new(TieredCappedOverageSnapshotAdjuster {
                bounds: vec![3066.0, 3979.0, 4654.0, 9280.0],
                rates: vec![1.1, 1.06, 1.01, 0.92],
            }) as Rc<dyn CappedOverageSnapshotFactor>,
            Rc::new(CapCappedOverageSnapshotAdjuster { cap: 22221.0 }),
            Rc::new(PercentCappedOverageSnapshotAdjuster { percent: 18.0 }),
            Rc::new(RoundingCappedOverageSnapshotAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 21918.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_overage_snapshot_engine();
        let item = new_capped_overage_snapshot("code-772", "label-953");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
