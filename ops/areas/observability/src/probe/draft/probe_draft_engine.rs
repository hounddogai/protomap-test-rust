use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the probe draft calculation.
pub trait ProbeDraftAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraftVersionOffsetAdjuster {
    pub weight: f64,
}

impl ProbeDraftAdjuster for ProbeDraftVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeDraft, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeDraftAdjuster for TieredProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeDraftAdjuster {
    pub cap: f64,
}

impl ProbeDraftAdjuster for CapProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeDraft, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeDraftAdjuster {
    pub percent: f64,
}

impl ProbeDraftAdjuster for PercentProbeDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the probe draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeDraftEngine {
    pub steps: Vec<Rc<dyn ProbeDraftAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeDraftEngine {
    pub fn price(&mut self, item: &ProbeDraft) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ProbeDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_probe_draft_engine() -> ProbeDraftEngine {
    ProbeDraftEngine {
        steps: vec![
            Rc::new(ProbeDraftVersionOffsetAdjuster { weight: 1.12 }) as Rc<dyn ProbeDraftAdjuster>,
            Rc::new(TieredProbeDraftAdjuster {
                bounds: vec![2127.0, 4163.0, 6848.0],
                rates: vec![1.16, 1.17, 1.05],
            }),
            Rc::new(CapProbeDraftAdjuster { cap: 2960.0 }),
            Rc::new(PercentProbeDraftAdjuster { percent: -6.4 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 47227.2,
    }
}
