use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the scheduled nonce score calculation.
pub trait ScheduledNonceScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledNonceScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledNonceScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledNonceScoreAdjuster for TieredScheduledNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledNonceScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameScheduledNonceScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledNonceScoreAdjuster for NameScheduledNonceScoreAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ScheduledNonceScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledNonceScoreAdjuster {
    pub percent: f64,
}

impl ScheduledNonceScoreAdjuster for PercentScheduledNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledNonceScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledNonceScoreAdjuster {
    pub minimum: f64,
}

impl ScheduledNonceScoreAdjuster for MinimumScheduledNonceScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledNonceScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled nonce score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledNonceScoreEngine {
    pub steps: Vec<Rc<dyn ScheduledNonceScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledNonceScoreEngine {
    pub fn compute(&mut self, item: &ScheduledNonceScore) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ScheduledNonceScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledNonceScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_nonce_score_engine() -> ScheduledNonceScoreEngine {
    ScheduledNonceScoreEngine {
        steps: vec![
            Rc::new(TieredScheduledNonceScoreAdjuster {
                bounds: vec![903.0, 5685.0, 5807.0, 9774.0],
                rates: vec![0.88, 0.96, 0.9, 0.9],
            }) as Rc<dyn ScheduledNonceScoreAdjuster>,
            Rc::new(NameScheduledNonceScoreAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.05), ("central".to_string(), 0.7)]),
            }),
            Rc::new(PercentScheduledNonceScoreAdjuster { percent: -15.5 }),
            Rc::new(MinimumScheduledNonceScoreAdjuster { minimum: 36.5 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 91761.2,
    }
}
