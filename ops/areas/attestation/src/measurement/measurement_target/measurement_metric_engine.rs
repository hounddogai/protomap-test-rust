use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the measurement metric calculation.
pub trait MeasurementMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedMeasurementMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl MeasurementMetricStep for FlaggedMeasurementMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &MeasurementMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementMetricPortOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementMetricStep for MeasurementMetricPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementMetric, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementMetricAdjuster {
    pub percent: f64,
}

impl MeasurementMetricStep for PercentMeasurementMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceMeasurementMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementMetricStep for SourceMeasurementMetricAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &MeasurementMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementMetricAdjuster {
    pub step: f64,
}

impl MeasurementMetricStep for RoundingMeasurementMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the measurement metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementMetricEngine {
    pub steps: Vec<Rc<dyn MeasurementMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementMetricEngine {
    pub fn compute(&mut self, item: &MeasurementMetric) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_metric_engine(flags: &flags::Registry) -> MeasurementMetricEngine {
    MeasurementMetricEngine {
        steps: vec![
            Rc::new(FlaggedMeasurementMetricAdjuster {
                flags: flags.clone(),
                flag: "measurement-metric-fallback".to_string(),
                percent: 5.2,
            }) as Rc<dyn MeasurementMetricStep>,
            Rc::new(MeasurementMetricPortOffsetAdjuster { weight: 1.37 }),
            Rc::new(PercentMeasurementMetricAdjuster { percent: 8.8 }),
            Rc::new(SourceMeasurementMetricAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.98),
                    ("north".to_string(), 1.24),
                    ("online".to_string(), 1.24),
                ]),
            }),
            Rc::new(RoundingMeasurementMetricAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 12002.6,
    }
}
