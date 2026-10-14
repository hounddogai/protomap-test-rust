use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the silence metric calculation.
pub trait SilenceMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceMetricQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl SilenceMetricFactor for SilenceMetricQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceMetric, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceMetricAdjuster {
    pub minimum: f64,
}

impl SilenceMetricFactor for MinimumSilenceMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceMetricAdjuster {
    pub percent: f64,
}

impl SilenceMetricFactor for PercentSilenceMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SilenceMetricFactor for TieredSilenceMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SilenceMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSilenceMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceMetricFactor for UnitSilenceMetricAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SilenceMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceMetricAdjuster {
    pub step: f64,
}

impl SilenceMetricFactor for RoundingSilenceMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the silence metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceMetricEngine {
    pub steps: Vec<Rc<dyn SilenceMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceMetricEngine {
    pub fn evaluate(&mut self, item: &SilenceMetric) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_silence_metric_engine() -> SilenceMetricEngine {
    SilenceMetricEngine {
        steps: vec![
            Rc::new(SilenceMetricQuotaUnitsOffsetAdjuster { weight: 0.52 }) as Rc<dyn SilenceMetricFactor>,
            Rc::new(MinimumSilenceMetricAdjuster { minimum: 34.8 }),
            Rc::new(PercentSilenceMetricAdjuster { percent: -12.0 }),
            Rc::new(TieredSilenceMetricAdjuster {
                bounds: vec![3600.0, 3852.0, 4888.0, 8121.0],
                rates: vec![1.1, 1.18, 1.08, 0.89],
            }),
            Rc::new(UnitSilenceMetricAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.2), ("north".to_string(), 0.73)]),
            }),
            Rc::new(RoundingSilenceMetricAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 58093.1,
    }
}
