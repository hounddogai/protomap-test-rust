use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped slo metric calculation.
pub trait CappedSloMetricAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSloMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleCappedSloMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CappedSloMetricAdjuster for TitleCappedSloMetricAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &CappedSloMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedSloMetricAdjuster {
    pub minimum: f64,
}

impl CappedSloMetricAdjuster for MinimumCappedSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedSloMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloMetricLimitOffsetAdjuster {
    pub weight: f64,
}

impl CappedSloMetricAdjuster for CappedSloMetricLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &CappedSloMetric, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedSloMetricAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedSloMetricAdjuster for FlaggedCappedSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedSloMetric, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedSloMetricAdjuster {
    pub percent: f64,
}

impl CappedSloMetricAdjuster for PercentCappedSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedSloMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedSloMetricAdjuster {
    pub step: f64,
}

impl CappedSloMetricAdjuster for RoundingCappedSloMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedSloMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the capped slo metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSloMetricEngine {
    pub steps: Vec<Rc<dyn CappedSloMetricAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSloMetricEngine {
    pub fn run(&mut self, item: &CappedSloMetric) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedSloMetricAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CappedSloMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_capped_slo_metric_engine(flags: &flags::Registry) -> CappedSloMetricEngine {
    CappedSloMetricEngine {
        steps: vec![
            Rc::new(TitleCappedSloMetricAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.11)]),
            }) as Rc<dyn CappedSloMetricAdjuster>,
            Rc::new(MinimumCappedSloMetricAdjuster { minimum: 38.6 }),
            Rc::new(CappedSloMetricLimitOffsetAdjuster { weight: 1.73 }),
            Rc::new(FlaggedCappedSloMetricAdjuster {
                flags: flags.clone(),
                flag: "capped-slo-metric-legacy".to_string(),
                percent: 9.8,
            }),
            Rc::new(PercentCappedSloMetricAdjuster { percent: -6.6 }),
            Rc::new(RoundingCappedSloMetricAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 46562.4,
    }
}
