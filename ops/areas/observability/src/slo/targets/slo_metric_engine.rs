use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo metric calculation.
pub trait SloMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMetricCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl SloMetricFactor for SloMetricCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &SloMetric, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloMetricFactor for FlaggedSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloMetricAdjuster {
    pub cap: f64,
}

impl SloMetricFactor for CapSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSloMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SloMetricFactor for NameSloMetricAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SloMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the slo metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloMetricEngine {
    pub steps: Vec<Rc<dyn SloMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloMetricEngine {
    pub fn evaluate(&mut self, item: &SloMetric) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[SloMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_slo_metric_engine(flags: &flags::Registry) -> SloMetricEngine {
    SloMetricEngine {
        steps: vec![
            Rc::new(SloMetricCreatedAtOffsetAdjuster { weight: 1.63 }) as Rc<dyn SloMetricFactor>,
            Rc::new(FlaggedSloMetricAdjuster {
                flags: flags.clone(),
                flag: "slo-metric-capped".to_string(),
                percent: -0.9,
            }),
            Rc::new(CapSloMetricAdjuster { cap: 3501.0 }),
            Rc::new(NameSloMetricAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.87),
                    ("online".to_string(), 1.09),
                    ("west".to_string(), 1.14),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 74907.3,
    }
}
