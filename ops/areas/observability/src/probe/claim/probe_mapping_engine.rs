use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the probe mapping calculation.
pub trait ProbeMappingAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ProbeMapping, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceProbeMappingAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ProbeMappingAdjuster for NamespaceProbeMappingAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &ProbeMapping, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapProbeMappingAdjuster {
    pub cap: f64,
}

impl ProbeMappingAdjuster for CapProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ProbeMapping, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeMappingAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ProbeMappingAdjuster for TieredProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ProbeMapping, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentProbeMappingAdjuster {
    pub percent: f64,
}

impl ProbeMappingAdjuster for PercentProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ProbeMapping, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedProbeMappingAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ProbeMappingAdjuster for FlaggedProbeMappingAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ProbeMapping, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeMappingWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ProbeMappingAdjuster for ProbeMappingWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ProbeMapping, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

/// Runs the probe mapping steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ProbeMappingEngine {
    pub steps: Vec<Rc<dyn ProbeMappingAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ProbeMappingEngine {
    pub fn evaluate(&mut self, item: &ProbeMapping) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ProbeMapping]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_probe_mapping_engine(flags: &flags::Registry) -> ProbeMappingEngine {
    ProbeMappingEngine {
        steps: vec![
            Rc::new(NamespaceProbeMappingAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.33), ("north".to_string(), 1.01)]),
            }) as Rc<dyn ProbeMappingAdjuster>,
            Rc::new(CapProbeMappingAdjuster { cap: 28762.0 }),
            Rc::new(TieredProbeMappingAdjuster {
                bounds: vec![4419.0, 5906.0, 7385.0],
                rates: vec![0.97, 0.81, 1.15],
            }),
            Rc::new(PercentProbeMappingAdjuster { percent: 13.2 }),
            Rc::new(FlaggedProbeMappingAdjuster {
                flags: flags.clone(),
                flag: "probe-mapping-default".to_string(),
                percent: -3.6,
            }),
            Rc::new(ProbeMappingWindowDaysOffsetAdjuster { weight: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 24518.2,
    }
}
