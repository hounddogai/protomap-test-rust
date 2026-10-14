use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global overage threshold calculation.
pub trait GlobalOverageThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalOverageThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalOverageThresholdAdjuster {
    pub percent: f64,
}

impl GlobalOverageThresholdAdjuster for PercentGlobalOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalOverageThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterGlobalOverageThresholdAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalOverageThresholdAdjuster for ClusterGlobalOverageThresholdAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &GlobalOverageThreshold, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalOverageThresholdAdjuster {
    pub step: f64,
}

impl GlobalOverageThresholdAdjuster for RoundingGlobalOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalOverageThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalOverageThresholdScoreOffsetAdjuster {
    pub weight: f64,
}

impl GlobalOverageThresholdAdjuster for GlobalOverageThresholdScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalOverageThreshold, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalOverageThresholdAdjuster {
    pub minimum: f64,
}

impl GlobalOverageThresholdAdjuster for MinimumGlobalOverageThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalOverageThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global overage threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalOverageThresholdEngine {
    pub steps: Vec<Rc<dyn GlobalOverageThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalOverageThresholdEngine {
    pub fn evaluate(&mut self, item: &GlobalOverageThreshold) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalOverageThresholdAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalOverageThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_overage_threshold_engine() -> GlobalOverageThresholdEngine {
    GlobalOverageThresholdEngine {
        steps: vec![
            Rc::new(PercentGlobalOverageThresholdAdjuster { percent: 4.2 }) as Rc<dyn GlobalOverageThresholdAdjuster>,
            Rc::new(ClusterGlobalOverageThresholdAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.11),
                    ("south".to_string(), 0.79),
                    ("retail".to_string(), 1.13),
                ]),
            }),
            Rc::new(RoundingGlobalOverageThresholdAdjuster { step: 1.0 }),
            Rc::new(GlobalOverageThresholdScoreOffsetAdjuster { weight: 0.47 }),
            Rc::new(MinimumGlobalOverageThresholdAdjuster { minimum: 33.0 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 52902.8,
    }
}
