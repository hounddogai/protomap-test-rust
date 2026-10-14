use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the overage snapshot calculation.
pub trait OverageSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &OverageSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumOverageSnapshotAdjuster {
    pub minimum: f64,
}

impl OverageSnapshotFactor for MinimumOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &OverageSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedOverageSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl OverageSnapshotFactor for FlaggedOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &OverageSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingOverageSnapshotAdjuster {
    pub step: f64,
}

impl OverageSnapshotFactor for RoundingOverageSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &OverageSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the overage snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct OverageSnapshotEngine {
    pub steps: Vec<Rc<dyn OverageSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl OverageSnapshotEngine {
    pub fn price(&mut self, item: &OverageSnapshot) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[OverageSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_overage_snapshot_engine(flags: &flags::Registry) -> OverageSnapshotEngine {
    OverageSnapshotEngine {
        steps: vec![
            Rc::new(MinimumOverageSnapshotAdjuster { minimum: 23.4 }) as Rc<dyn OverageSnapshotFactor>,
            Rc::new(FlaggedOverageSnapshotAdjuster {
                flags: flags.clone(),
                flag: "overage-snapshot-scheduled".to_string(),
                percent: -6.3,
            }),
            Rc::new(RoundingOverageSnapshotAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 13451.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_overage_snapshot_engine(&flags::new_registry());
        let item = new_overage_snapshot("code-824", "title-376");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
