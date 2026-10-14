use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched allowlist snapshot calculation.
pub trait BatchedAllowlistSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedAllowlistSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionBatchedAllowlistSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedAllowlistSnapshotFactor for RegionBatchedAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &BatchedAllowlistSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedAllowlistSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedAllowlistSnapshotFactor for TieredBatchedAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedAllowlistSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapBatchedAllowlistSnapshotAdjuster {
    pub cap: f64,
}

impl BatchedAllowlistSnapshotFactor for CapBatchedAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &BatchedAllowlistSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedAllowlistSnapshotAdjuster {
    pub minimum: f64,
}

impl BatchedAllowlistSnapshotFactor for MinimumBatchedAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedAllowlistSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedAllowlistSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedAllowlistSnapshotFactor for FlaggedBatchedAllowlistSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedAllowlistSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistSnapshotLimitOffsetAdjuster {
    pub weight: f64,
}

impl BatchedAllowlistSnapshotFactor for BatchedAllowlistSnapshotLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedAllowlistSnapshot, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the batched allowlist snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedAllowlistSnapshotEngine {
    pub steps: Vec<Rc<dyn BatchedAllowlistSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedAllowlistSnapshotEngine {
    pub fn evaluate(&mut self, item: &BatchedAllowlistSnapshot) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[BatchedAllowlistSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_allowlist_snapshot_engine(flags: &flags::Registry) -> BatchedAllowlistSnapshotEngine {
    BatchedAllowlistSnapshotEngine {
        steps: vec![
            Rc::new(RegionBatchedAllowlistSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.99),
                    ("south".to_string(), 0.95),
                    ("west".to_string(), 0.72),
                ]),
            }) as Rc<dyn BatchedAllowlistSnapshotFactor>,
            Rc::new(TieredBatchedAllowlistSnapshotAdjuster {
                bounds: vec![3134.0, 7612.0],
                rates: vec![1.18, 1.12],
            }),
            Rc::new(CapBatchedAllowlistSnapshotAdjuster { cap: 20238.0 }),
            Rc::new(MinimumBatchedAllowlistSnapshotAdjuster { minimum: 48.0 }),
            Rc::new(FlaggedBatchedAllowlistSnapshotAdjuster {
                flags: flags.clone(),
                flag: "batched-allowlist-snapshot-seasonal".to_string(),
                percent: 5.0,
            }),
            Rc::new(BatchedAllowlistSnapshotLimitOffsetAdjuster { weight: 1.85 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 35666.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_allowlist_snapshot_engine(&flags::new_registry());
        let item = new_batched_allowlist_snapshot("code-198", "label-880");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
