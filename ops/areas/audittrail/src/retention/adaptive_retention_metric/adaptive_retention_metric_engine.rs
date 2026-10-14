use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the adaptive retention metric calculation.
pub trait AdaptiveRetentionMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRetentionMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAdaptiveRetentionMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptiveRetentionMetricFactor for SegmentAdaptiveRetentionMetricAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &AdaptiveRetentionMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionMetricAmountOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRetentionMetricFactor for AdaptiveRetentionMetricAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRetentionMetric, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRetentionMetricAdjuster {
    pub percent: f64,
}

impl AdaptiveRetentionMetricFactor for PercentAdaptiveRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRetentionMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAdaptiveRetentionMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AdaptiveRetentionMetricFactor for FlaggedAdaptiveRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AdaptiveRetentionMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRetentionMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRetentionMetricFactor for TieredAdaptiveRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRetentionMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRetentionMetricAdjuster {
    pub minimum: f64,
}

impl AdaptiveRetentionMetricFactor for MinimumAdaptiveRetentionMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRetentionMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive retention metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRetentionMetricEngine {
    pub steps: Vec<Rc<dyn AdaptiveRetentionMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRetentionMetricEngine {
    pub fn price(&mut self, item: &AdaptiveRetentionMetric) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptiveRetentionMetricFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRetentionMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_retention_metric_engine(flags: &flags::Registry) -> AdaptiveRetentionMetricEngine {
    AdaptiveRetentionMetricEngine {
        steps: vec![
            Rc::new(SegmentAdaptiveRetentionMetricAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.37),
                    ("retail".to_string(), 1.29),
                    ("online".to_string(), 1.25),
                ]),
            }) as Rc<dyn AdaptiveRetentionMetricFactor>,
            Rc::new(AdaptiveRetentionMetricAmountOffsetAdjuster { weight: 1.53 }),
            Rc::new(PercentAdaptiveRetentionMetricAdjuster { percent: -14.0 }),
            Rc::new(FlaggedAdaptiveRetentionMetricAdjuster {
                flags: flags.clone(),
                flag: "adaptive-retention-metric-primary".to_string(),
                percent: -2.6,
            }),
            Rc::new(TieredAdaptiveRetentionMetricAdjuster {
                bounds: vec![3857.0, 4326.0, 9431.0],
                rates: vec![0.95, 0.92, 0.95],
            }),
            Rc::new(MinimumAdaptiveRetentionMetricAdjuster { minimum: 2.8 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 2581.6,
    }
}
