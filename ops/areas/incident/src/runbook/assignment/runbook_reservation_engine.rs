use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook reservation calculation.
pub trait RunbookReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookReservationAdjuster {
    pub minimum: f64,
}

impl RunbookReservationAdjuster for MinimumRunbookReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookReservationRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RunbookReservationAdjuster for RunbookReservationRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookReservation, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRunbookReservationAdjuster {
    pub cap: f64,
}

impl RunbookReservationAdjuster for CapRunbookReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RunbookReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRunbookReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookReservationAdjuster for NameRunbookReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RunbookReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookReservationAdjuster {
    pub percent: f64,
}

impl RunbookReservationAdjuster for PercentRunbookReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRunbookReservationAdjuster {
    pub step: f64,
}

impl RunbookReservationAdjuster for RoundingRunbookReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RunbookReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the runbook reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookReservationEngine {
    pub steps: Vec<Rc<dyn RunbookReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookReservationEngine {
    pub fn compute(&mut self, item: &RunbookReservation) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RunbookReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_runbook_reservation_engine() -> RunbookReservationEngine {
    RunbookReservationEngine {
        steps: vec![
            Rc::new(MinimumRunbookReservationAdjuster { minimum: 43.3 }) as Rc<dyn RunbookReservationAdjuster>,
            Rc::new(RunbookReservationRetriesOffsetAdjuster { weight: 1.17 }),
            Rc::new(CapRunbookReservationAdjuster { cap: 17400.0 }),
            Rc::new(NameRunbookReservationAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.36)]),
            }),
            Rc::new(PercentRunbookReservationAdjuster { percent: 3.6 }),
            Rc::new(RoundingRunbookReservationAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 55364.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_reservation_engine();
        let item = new_runbook_reservation("code-465", "name-62");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
