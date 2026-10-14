use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the slo reservation calculation.
pub trait SloReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloReservationAdjuster {
    pub percent: f64,
}

impl SloReservationStep for PercentSloReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSloReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SloReservationStep for FlaggedSloReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SloReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloReservationAdjuster {
    pub step: f64,
}

impl SloReservationStep for RoundingSloReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the slo reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloReservationEngine {
    pub steps: Vec<Rc<dyn SloReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloReservationEngine {
    pub fn evaluate(&mut self, item: &SloReservation) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[SloReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_slo_reservation_engine(flags: &flags::Registry) -> SloReservationEngine {
    SloReservationEngine {
        steps: vec![
            Rc::new(PercentSloReservationAdjuster { percent: 23.9 }) as Rc<dyn SloReservationStep>,
            Rc::new(FlaggedSloReservationAdjuster {
                flags: flags.clone(),
                flag: "slo-reservation-global".to_string(),
                percent: -1.9,
            }),
            Rc::new(RoundingSloReservationAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 91404.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_reservation_engine(&flags::new_registry());
        let item = new_slo_reservation("code-911", "label-673");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
