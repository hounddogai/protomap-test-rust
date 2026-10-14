use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the lenient rule reservation calculation.
pub trait LenientRuleReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientRuleReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientRuleReservationAdjuster {
    pub minimum: f64,
}

impl LenientRuleReservationFactor for MinimumLenientRuleReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientRuleReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLenientRuleReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientRuleReservationFactor for TitleLenientRuleReservationAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &LenientRuleReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRuleReservationWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl LenientRuleReservationFactor for LenientRuleReservationWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &LenientRuleReservation, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientRuleReservationAdjuster {
    pub cap: f64,
}

impl LenientRuleReservationFactor for CapLenientRuleReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientRuleReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientRuleReservationAdjuster {
    pub step: f64,
}

impl LenientRuleReservationFactor for RoundingLenientRuleReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientRuleReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientRuleReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientRuleReservationFactor for TieredLenientRuleReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientRuleReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient rule reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientRuleReservationEngine {
    pub steps: Vec<Rc<dyn LenientRuleReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientRuleReservationEngine {
    pub fn compute(&mut self, item: &LenientRuleReservation) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientRuleReservationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientRuleReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_rule_reservation_engine() -> LenientRuleReservationEngine {
    LenientRuleReservationEngine {
        steps: vec![
            Rc::new(MinimumLenientRuleReservationAdjuster { minimum: 2.4 }) as Rc<dyn LenientRuleReservationFactor>,
            Rc::new(TitleLenientRuleReservationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.3)]),
            }),
            Rc::new(LenientRuleReservationWindowDaysOffsetAdjuster { weight: 0.3 }),
            Rc::new(CapLenientRuleReservationAdjuster { cap: 21572.0 }),
            Rc::new(RoundingLenientRuleReservationAdjuster { step: 0.05 }),
            Rc::new(TieredLenientRuleReservationAdjuster {
                bounds: vec![571.0, 1024.0, 8387.0],
                rates: vec![0.86, 1.02, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 27403.8,
    }
}
