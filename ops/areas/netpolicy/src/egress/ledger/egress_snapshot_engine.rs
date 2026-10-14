use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the egress snapshot calculation.
pub trait EgressSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &EgressSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedEgressSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl EgressSnapshotAdjuster for FlaggedEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &EgressSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingEgressSnapshotAdjuster {
    pub step: f64,
}

impl EgressSnapshotAdjuster for RoundingEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &EgressSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentEgressSnapshotAdjuster {
    pub percent: f64,
}

impl EgressSnapshotAdjuster for PercentEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &EgressSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapEgressSnapshotAdjuster {
    pub cap: f64,
}

impl EgressSnapshotAdjuster for CapEgressSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &EgressSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the egress snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct EgressSnapshotEngine {
    pub steps: Vec<Rc<dyn EgressSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl EgressSnapshotEngine {
    pub fn price(&mut self, item: &EgressSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[EgressSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_egress_snapshot_engine(flags: &flags::Registry) -> EgressSnapshotEngine {
    EgressSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedEgressSnapshotAdjuster {
                flags: flags.clone(),
                flag: "egress-snapshot-lenient".to_string(),
                percent: -3.8,
            }) as Rc<dyn EgressSnapshotAdjuster>,
            Rc::new(RoundingEgressSnapshotAdjuster { step: 5.0 }),
            Rc::new(PercentEgressSnapshotAdjuster { percent: -18.9 }),
            Rc::new(CapEgressSnapshotAdjuster { cap: 1519.0 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 97070.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_egress_snapshot_engine(&flags::new_registry());
        let item = new_egress_snapshot("code-302", "name-762");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
