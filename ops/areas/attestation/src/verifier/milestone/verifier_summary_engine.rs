use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the verifier summary calculation.
pub trait VerifierSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &VerifierSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapVerifierSummaryAdjuster {
    pub cap: f64,
}

impl VerifierSummaryStep for CapVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &VerifierSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleVerifierSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl VerifierSummaryStep for TitleVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &VerifierSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl VerifierSummaryStep for TieredVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &VerifierSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingVerifierSummaryAdjuster {
    pub step: f64,
}

impl VerifierSummaryStep for RoundingVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &VerifierSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumVerifierSummaryAdjuster {
    pub minimum: f64,
}

impl VerifierSummaryStep for MinimumVerifierSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &VerifierSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the verifier summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct VerifierSummaryEngine {
    pub steps: Vec<Rc<dyn VerifierSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl VerifierSummaryEngine {
    pub fn run(&mut self, item: &VerifierSummary) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn VerifierSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[VerifierSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_verifier_summary_engine() -> VerifierSummaryEngine {
    VerifierSummaryEngine {
        steps: vec![
            Rc::new(CapVerifierSummaryAdjuster { cap: 44798.0 }) as Rc<dyn VerifierSummaryStep>,
            Rc::new(TitleVerifierSummaryAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.17), ("online".to_string(), 1.16)]),
            }),
            Rc::new(TieredVerifierSummaryAdjuster {
                bounds: vec![7913.0, 9286.0],
                rates: vec![0.93, 0.94],
            }),
            Rc::new(RoundingVerifierSummaryAdjuster { step: 1.0 }),
            Rc::new(MinimumVerifierSummaryAdjuster { minimum: 33.1 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 51593.7,
    }
}
