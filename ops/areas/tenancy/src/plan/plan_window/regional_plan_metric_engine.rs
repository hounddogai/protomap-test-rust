use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional plan metric calculation.
pub trait RegionalPlanMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalPlanMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanMetricBurstOffsetAdjuster {
    pub weight: f64,
}

impl RegionalPlanMetricModifier for RegionalPlanMetricBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalPlanMetric, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRegionalPlanMetricAdjuster {
    pub step: f64,
}

impl RegionalPlanMetricModifier for RoundingRegionalPlanMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RegionalPlanMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalPlanMetricAdjuster {
    pub cap: f64,
}

impl RegionalPlanMetricModifier for CapRegionalPlanMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalPlanMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalPlanMetricAdjuster {
    pub percent: f64,
}

impl RegionalPlanMetricModifier for PercentRegionalPlanMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalPlanMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRegionalPlanMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalPlanMetricModifier for CountryRegionalPlanMetricAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RegionalPlanMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional plan metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalPlanMetricEngine {
    pub steps: Vec<Rc<dyn RegionalPlanMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalPlanMetricEngine {
    pub fn price(&mut self, item: &RegionalPlanMetric) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RegionalPlanMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RegionalPlanMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_regional_plan_metric_engine() -> RegionalPlanMetricEngine {
    RegionalPlanMetricEngine {
        steps: vec![
            Rc::new(RegionalPlanMetricBurstOffsetAdjuster { weight: 1.62 }) as Rc<dyn RegionalPlanMetricModifier>,
            Rc::new(RoundingRegionalPlanMetricAdjuster { step: 1.0 }),
            Rc::new(CapRegionalPlanMetricAdjuster { cap: 47765.0 }),
            Rc::new(PercentRegionalPlanMetricAdjuster { percent: -1.8 }),
            Rc::new(CountryRegionalPlanMetricAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.92),
                    ("east".to_string(), 1.13),
                    ("central".to_string(), 1.29),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 39927.7,
    }
}
