use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient token reservation calculation.
pub trait LenientTokenReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientTokenReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientTokenReservationAdjuster {
    pub cap: f64,
}

impl LenientTokenReservationAdjuster for CapLenientTokenReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientTokenReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientTokenReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientTokenReservationAdjuster for FlaggedLenientTokenReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientTokenReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientTokenReservationAdjuster {
    pub step: f64,
}

impl LenientTokenReservationAdjuster for RoundingLenientTokenReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientTokenReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientTokenReservationAdjuster {
    pub minimum: f64,
}

impl LenientTokenReservationAdjuster for MinimumLenientTokenReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientTokenReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelLenientTokenReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl LenientTokenReservationAdjuster for LabelLenientTokenReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &LenientTokenReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientTokenReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientTokenReservationAdjuster for TieredLenientTokenReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientTokenReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient token reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientTokenReservationEngine {
    pub steps: Vec<Rc<dyn LenientTokenReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientTokenReservationEngine {
    pub fn run(&mut self, item: &LenientTokenReservation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientTokenReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_token_reservation_engine(flags: &flags::Registry) -> LenientTokenReservationEngine {
    LenientTokenReservationEngine {
        steps: vec![
            Rc::new(CapLenientTokenReservationAdjuster { cap: 35824.0 }) as Rc<dyn LenientTokenReservationAdjuster>,
            Rc::new(FlaggedLenientTokenReservationAdjuster {
                flags: flags.clone(),
                flag: "lenient-token-reservation-default".to_string(),
                percent: -6.4,
            }),
            Rc::new(RoundingLenientTokenReservationAdjuster { step: 1.0 }),
            Rc::new(MinimumLenientTokenReservationAdjuster { minimum: 30.7 }),
            Rc::new(LabelLenientTokenReservationAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.29), ("south".to_string(), 0.93)]),
            }),
            Rc::new(TieredLenientTokenReservationAdjuster {
                bounds: vec![1447.0, 2272.0, 9135.0],
                rates: vec![1.16, 1.2, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 12353.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_token_reservation_engine(&flags::new_registry());
        let item = new_lenient_token_reservation("code-234", "label-490");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
