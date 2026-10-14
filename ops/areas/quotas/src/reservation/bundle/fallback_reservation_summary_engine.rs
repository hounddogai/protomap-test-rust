use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback reservation summary calculation.
pub trait FallbackReservationSummaryStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackReservationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleFallbackReservationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackReservationSummaryStep for LocaleFallbackReservationSummaryAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &FallbackReservationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackReservationSummaryAdjuster {
    pub step: f64,
}

impl FallbackReservationSummaryStep for RoundingFallbackReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackReservationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackReservationSummaryAdjuster {
    pub minimum: f64,
}

impl FallbackReservationSummaryStep for MinimumFallbackReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackReservationSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the fallback reservation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackReservationSummaryEngine {
    pub steps: Vec<Rc<dyn FallbackReservationSummaryStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackReservationSummaryEngine {
    pub fn compute(&mut self, item: &FallbackReservationSummary) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackReservationSummaryStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackReservationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_reservation_summary_engine() -> FallbackReservationSummaryEngine {
    FallbackReservationSummaryEngine {
        steps: vec![
            Rc::new(LocaleFallbackReservationSummaryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.17)]),
            }) as Rc<dyn FallbackReservationSummaryStep>,
            Rc::new(RoundingFallbackReservationSummaryAdjuster { step: 0.25 }),
            Rc::new(MinimumFallbackReservationSummaryAdjuster { minimum: 20.4 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 91980.5,
    }
}
