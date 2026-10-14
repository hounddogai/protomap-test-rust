use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary forecast score calculation.
pub trait PrimaryForecastScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryForecastScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryForecastScoreAdjuster {
    pub minimum: f64,
}

impl PrimaryForecastScoreAdjuster for MinimumPrimaryForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryForecastScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryForecastScoreAdjuster {
    pub step: f64,
}

impl PrimaryForecastScoreAdjuster for RoundingPrimaryForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryForecastScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryForecastScoreAdjuster {
    pub percent: f64,
}

impl PrimaryForecastScoreAdjuster for PercentPrimaryForecastScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryForecastScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryForecastScoreScoreOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryForecastScoreAdjuster for PrimaryForecastScoreScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryForecastScore, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPrimaryForecastScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryForecastScoreAdjuster for ClusterPrimaryForecastScoreAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PrimaryForecastScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary forecast score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryForecastScoreEngine {
    pub steps: Vec<Rc<dyn PrimaryForecastScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryForecastScoreEngine {
    pub fn price(&mut self, item: &PrimaryForecastScore) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryForecastScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_forecast_score_engine() -> PrimaryForecastScoreEngine {
    PrimaryForecastScoreEngine {
        steps: vec![
            Rc::new(MinimumPrimaryForecastScoreAdjuster { minimum: 38.0 }) as Rc<dyn PrimaryForecastScoreAdjuster>,
            Rc::new(RoundingPrimaryForecastScoreAdjuster { step: 0.1 }),
            Rc::new(PercentPrimaryForecastScoreAdjuster { percent: -19.0 }),
            Rc::new(PrimaryForecastScoreScoreOffsetAdjuster { weight: 1.94 }),
            Rc::new(ClusterPrimaryForecastScoreAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.75), ("north".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 43646.6,
    }
}
