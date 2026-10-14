use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the default forecast segment calculation.
pub trait DefaultForecastSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultForecastSegment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultForecastSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultForecastSegmentStep for FlaggedDefaultForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultForecastSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultForecastSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultForecastSegmentStep for TieredDefaultForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultForecastSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultForecastSegmentAdjuster {
    pub percent: f64,
}

impl DefaultForecastSegmentStep for PercentDefaultForecastSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultForecastSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDefaultForecastSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultForecastSegmentStep for NamespaceDefaultForecastSegmentAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &DefaultForecastSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the default forecast segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultForecastSegmentEngine {
    pub steps: Vec<Rc<dyn DefaultForecastSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultForecastSegmentEngine {
    pub fn compute(&mut self, item: &DefaultForecastSegment) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultForecastSegmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultForecastSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_forecast_segment_engine(flags: &flags::Registry) -> DefaultForecastSegmentEngine {
    DefaultForecastSegmentEngine {
        steps: vec![
            Rc::new(FlaggedDefaultForecastSegmentAdjuster {
                flags: flags.clone(),
                flag: "default-forecast-segment-primary".to_string(),
                percent: -1.1,
            }) as Rc<dyn DefaultForecastSegmentStep>,
            Rc::new(TieredDefaultForecastSegmentAdjuster {
                bounds: vec![3406.0, 6045.0, 7162.0],
                rates: vec![0.9, 0.98, 1.03],
            }),
            Rc::new(PercentDefaultForecastSegmentAdjuster { percent: 14.9 }),
            Rc::new(NamespaceDefaultForecastSegmentAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.76), ("west".to_string(), 1.27)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 70382.5,
    }
}
