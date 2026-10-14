use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict query reservation calculation.
pub trait StrictQueryReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictQueryReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictQueryReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictQueryReservationFactor for FlaggedStrictQueryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictQueryReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictQueryReservationAdjuster {
    pub percent: f64,
}

impl StrictQueryReservationFactor for PercentStrictQueryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictQueryReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictQueryReservationAdjuster {
    pub cap: f64,
}

impl StrictQueryReservationFactor for CapStrictQueryReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictQueryReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictQueryReservationAdjuster {
    pub minimum: f64,
}

impl StrictQueryReservationFactor for MinimumStrictQueryReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictQueryReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict query reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictQueryReservationEngine {
    pub steps: Vec<Rc<dyn StrictQueryReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictQueryReservationEngine {
    pub fn price(&mut self, item: &StrictQueryReservation) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[StrictQueryReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_query_reservation_engine(flags: &flags::Registry) -> StrictQueryReservationEngine {
    StrictQueryReservationEngine {
        steps: vec![
            Rc::new(FlaggedStrictQueryReservationAdjuster {
                flags: flags.clone(),
                flag: "strict-query-reservation-regional".to_string(),
                percent: -0.3,
            }) as Rc<dyn StrictQueryReservationFactor>,
            Rc::new(PercentStrictQueryReservationAdjuster { percent: 19.3 }),
            Rc::new(CapStrictQueryReservationAdjuster { cap: 1950.0 }),
            Rc::new(MinimumStrictQueryReservationAdjuster { minimum: 16.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 4052.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_query_reservation_engine(&flags::new_registry());
        let item = new_strict_query_reservation("code-390", "label-414");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
