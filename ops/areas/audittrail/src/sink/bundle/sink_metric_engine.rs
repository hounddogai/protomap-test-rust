use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink metric calculation.
pub trait SinkMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkMetricFactor for FlaggedSinkMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSinkMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkMetricFactor for SegmentSinkMetricAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &SinkMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkMetricAdjuster {
    pub percent: f64,
}

impl SinkMetricFactor for PercentSinkMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the sink metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkMetricEngine {
    pub steps: Vec<Rc<dyn SinkMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkMetricEngine {
    pub fn compute(&mut self, item: &SinkMetric) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[SinkMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_sink_metric_engine(flags: &flags::Registry) -> SinkMetricEngine {
    SinkMetricEngine {
        steps: vec![
            Rc::new(FlaggedSinkMetricAdjuster {
                flags: flags.clone(),
                flag: "sink-metric-strict".to_string(),
                percent: 1.1,
            }) as Rc<dyn SinkMetricFactor>,
            Rc::new(SegmentSinkMetricAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.88), ("retail".to_string(), 1.2)]),
            }),
            Rc::new(PercentSinkMetricAdjuster { percent: 13.2 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 70904.5,
    }
}
