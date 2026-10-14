use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe baseline calculation.
pub trait ProbeBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeBaselineAdjuster {
    pub percent: f64,
}

impl ProbeBaselineModifier for PercentProbeBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeBaselineAdjuster {
    pub cap: f64,
}

impl ProbeBaselineModifier for CapProbeBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeBaselineModifier for TieredProbeBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the probe baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeBaselineEngine {
    pub steps: Vec<Rc<dyn ProbeBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeBaselineEngine {
    pub fn run(&mut self, item: &ProbeBaseline) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ProbeBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_probe_baseline_engine() -> ProbeBaselineEngine {
    ProbeBaselineEngine {
        steps: vec![
            Rc::new(PercentProbeBaselineAdjuster { percent: -22.8 }) as Rc<dyn ProbeBaselineModifier>,
            Rc::new(CapProbeBaselineAdjuster { cap: 44159.0 }),
            Rc::new(TieredProbeBaselineAdjuster {
                bounds: vec![7093.0, 7658.0, 8149.0, 9043.0],
                rates: vec![1.03, 1.13, 0.93, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 38788.9,
    }
}
