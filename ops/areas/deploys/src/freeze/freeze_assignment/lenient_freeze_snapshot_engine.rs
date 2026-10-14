use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the lenient freeze snapshot calculation.
pub trait LenientFreezeSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientFreezeSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientFreezeSnapshotAdjuster {
    pub cap: f64,
}

impl LenientFreezeSnapshotModifier for CapLenientFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientFreezeSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientFreezeSnapshotBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl LenientFreezeSnapshotModifier for LenientFreezeSnapshotBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &LenientFreezeSnapshot, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientFreezeSnapshotAdjuster {
    pub minimum: f64,
}

impl LenientFreezeSnapshotModifier for MinimumLenientFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientFreezeSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientFreezeSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientFreezeSnapshotModifier for FlaggedLenientFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientFreezeSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientFreezeSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientFreezeSnapshotModifier for TieredLenientFreezeSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientFreezeSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient freeze snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientFreezeSnapshotEngine {
    pub steps: Vec<Rc<dyn LenientFreezeSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientFreezeSnapshotEngine {
    pub fn price(&mut self, item: &LenientFreezeSnapshot) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientFreezeSnapshotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientFreezeSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_lenient_freeze_snapshot_engine(flags: &flags::Registry) -> LenientFreezeSnapshotEngine {
    LenientFreezeSnapshotEngine {
        steps: vec![
            Rc::new(CapLenientFreezeSnapshotAdjuster { cap: 7876.0 }) as Rc<dyn LenientFreezeSnapshotModifier>,
            Rc::new(LenientFreezeSnapshotBatchSizeOffsetAdjuster { weight: 1.68 }),
            Rc::new(MinimumLenientFreezeSnapshotAdjuster { minimum: 7.9 }),
            Rc::new(FlaggedLenientFreezeSnapshotAdjuster {
                flags: flags.clone(),
                flag: "lenient-freeze-snapshot-weighted".to_string(),
                percent: 5.1,
            }),
            Rc::new(TieredLenientFreezeSnapshotAdjuster {
                bounds: vec![3293.0, 8949.0],
                rates: vec![0.85, 0.86],
            }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 71124.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_freeze_snapshot_engine(&flags::new_registry());
        let item = new_lenient_freeze_snapshot("code-28", "title-218");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
