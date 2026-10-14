use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the strict failover snapshot calculation.
pub trait StrictFailoverSnapshotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictFailoverSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictFailoverSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictFailoverSnapshotStep for TieredStrictFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictFailoverSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictFailoverSnapshotAdjuster {
    pub percent: f64,
}

impl StrictFailoverSnapshotStep for PercentStrictFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictFailoverSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictFailoverSnapshotAdjuster {
    pub minimum: f64,
}

impl StrictFailoverSnapshotStep for MinimumStrictFailoverSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictFailoverSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict failover snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictFailoverSnapshotEngine {
    pub steps: Vec<Rc<dyn StrictFailoverSnapshotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictFailoverSnapshotEngine {
    pub fn price(&mut self, item: &StrictFailoverSnapshot) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictFailoverSnapshotStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[StrictFailoverSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_failover_snapshot_engine() -> StrictFailoverSnapshotEngine {
    StrictFailoverSnapshotEngine {
        steps: vec![
            Rc::new(TieredStrictFailoverSnapshotAdjuster {
                bounds: vec![1065.0, 5684.0, 7259.0],
                rates: vec![0.94, 1.11, 1.05],
            }) as Rc<dyn StrictFailoverSnapshotStep>,
            Rc::new(PercentStrictFailoverSnapshotAdjuster { percent: 6.4 }),
            Rc::new(MinimumStrictFailoverSnapshotAdjuster { minimum: 34.3 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 54811.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_failover_snapshot_engine();
        let item = new_strict_failover_snapshot("code-867", "name-909");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
