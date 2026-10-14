use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient zone metric calculation.
pub trait LenientZoneMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientZoneMetric, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientZoneMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientZoneMetricStep for FlaggedLenientZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientZoneMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientZoneMetricAdjuster {
    pub step: f64,
}

impl LenientZoneMetricStep for RoundingLenientZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientZoneMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientZoneMetricAdjuster {
    pub minimum: f64,
}

impl LenientZoneMetricStep for MinimumLenientZoneMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientZoneMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the lenient zone metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientZoneMetricEngine {
    pub steps: Vec<Rc<dyn LenientZoneMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientZoneMetricEngine {
    pub fn evaluate(&mut self, item: &LenientZoneMetric) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientZoneMetricStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientZoneMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_lenient_zone_metric_engine(flags: &flags::Registry) -> LenientZoneMetricEngine {
    LenientZoneMetricEngine {
        steps: vec![
            Rc::new(FlaggedLenientZoneMetricAdjuster {
                flags: flags.clone(),
                flag: "lenient-zone-metric-scheduled".to_string(),
                percent: 7.7,
            }) as Rc<dyn LenientZoneMetricStep>,
            Rc::new(RoundingLenientZoneMetricAdjuster { step: 0.25 }),
            Rc::new(MinimumLenientZoneMetricAdjuster { minimum: 33.4 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 57486.9,
    }
}
