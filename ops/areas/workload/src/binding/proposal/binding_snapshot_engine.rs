use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding snapshot calculation.
pub trait BindingSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingSnapshotRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl BindingSnapshotFactor for BindingSnapshotRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &BindingSnapshot, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingSnapshotAdjuster {
    pub minimum: f64,
}

impl BindingSnapshotFactor for MinimumBindingSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelBindingSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingSnapshotFactor for LabelBindingSnapshotAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &BindingSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingSnapshotAdjuster {
    pub step: f64,
}

impl BindingSnapshotFactor for RoundingBindingSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingSnapshotAdjuster {
    pub percent: f64,
}

impl BindingSnapshotFactor for PercentBindingSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the binding snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingSnapshotEngine {
    pub steps: Vec<Rc<dyn BindingSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingSnapshotEngine {
    pub fn evaluate(&mut self, item: &BindingSnapshot) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BindingSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_binding_snapshot_engine() -> BindingSnapshotEngine {
    BindingSnapshotEngine {
        steps: vec![
            Rc::new(BindingSnapshotRotationDaysOffsetAdjuster { weight: 1.85 }) as Rc<dyn BindingSnapshotFactor>,
            Rc::new(MinimumBindingSnapshotAdjuster { minimum: 14.4 }),
            Rc::new(LabelBindingSnapshotAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.82), ("north".to_string(), 1.12)]),
            }),
            Rc::new(RoundingBindingSnapshotAdjuster { step: 5.0 }),
            Rc::new(PercentBindingSnapshotAdjuster { percent: -15.3 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 12695.5,
    }
}
