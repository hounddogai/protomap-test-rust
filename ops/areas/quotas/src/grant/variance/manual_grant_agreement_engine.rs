use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual grant agreement calculation.
pub trait ManualGrantAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualGrantAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualGrantAgreementAdjuster {
    pub step: f64,
}

impl ManualGrantAgreementFactor for RoundingManualGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualGrantAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualGrantAgreementAdjuster {
    pub minimum: f64,
}

impl ManualGrantAgreementFactor for MinimumManualGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualGrantAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualGrantAgreementAdjuster {
    pub percent: f64,
}

impl ManualGrantAgreementFactor for PercentManualGrantAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualGrantAgreement, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the manual grant agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualGrantAgreementEngine {
    pub steps: Vec<Rc<dyn ManualGrantAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualGrantAgreementEngine {
    pub fn evaluate(&mut self, item: &ManualGrantAgreement) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualGrantAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualGrantAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_manual_grant_agreement_engine() -> ManualGrantAgreementEngine {
    ManualGrantAgreementEngine {
        steps: vec![
            Rc::new(RoundingManualGrantAgreementAdjuster { step: 0.05 }) as Rc<dyn ManualGrantAgreementFactor>,
            Rc::new(MinimumManualGrantAgreementAdjuster { minimum: 28.9 }),
            Rc::new(PercentManualGrantAgreementAdjuster { percent: -18.2 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 31657.3,
    }
}
