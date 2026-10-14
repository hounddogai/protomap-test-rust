use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the trust domain score calculation.
pub trait TrustDomainScoreAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainScoreAdjuster {
    pub percent: f64,
}

impl TrustDomainScoreAdjuster for PercentTrustDomainScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainScoreAdjuster {
    pub step: f64,
}

impl TrustDomainScoreAdjuster for RoundingTrustDomainScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainScoreAdjuster {
    pub cap: f64,
}

impl TrustDomainScoreAdjuster for CapTrustDomainScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainScoreAdjuster {
    pub minimum: f64,
}

impl TrustDomainScoreAdjuster for MinimumTrustDomainScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the trust domain score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainScoreEngine {
    pub steps: Vec<Rc<dyn TrustDomainScoreAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainScoreEngine {
    pub fn compute(&mut self, item: &TrustDomainScore) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainScoreAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_score_engine() -> TrustDomainScoreEngine {
    TrustDomainScoreEngine {
        steps: vec![
            Rc::new(PercentTrustDomainScoreAdjuster { percent: -6.1 }) as Rc<dyn TrustDomainScoreAdjuster>,
            Rc::new(RoundingTrustDomainScoreAdjuster { step: 0.05 }),
            Rc::new(CapTrustDomainScoreAdjuster { cap: 6010.0 }),
            Rc::new(MinimumTrustDomainScoreAdjuster { minimum: 36.7 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 50516.5,
    }
}
