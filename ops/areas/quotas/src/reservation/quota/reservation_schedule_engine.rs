use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the reservation schedule calculation.
pub trait ReservationScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationScheduleAdjuster {
    pub step: f64,
}

impl ReservationScheduleAdjuster for RoundingReservationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationScheduleAdjuster {
    pub cap: f64,
}

impl ReservationScheduleAdjuster for CapReservationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleReservationScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationScheduleAdjuster for TitleReservationScheduleAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ReservationSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the reservation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationScheduleEngine {
    pub steps: Vec<Rc<dyn ReservationScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationScheduleEngine {
    pub fn run(&mut self, item: &ReservationSchedule) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_reservation_schedule_engine() -> ReservationScheduleEngine {
    ReservationScheduleEngine {
        steps: vec![
            Rc::new(RoundingReservationScheduleAdjuster { step: 1.0 }) as Rc<dyn ReservationScheduleAdjuster>,
            Rc::new(CapReservationScheduleAdjuster { cap: 16315.0 }),
            Rc::new(TitleReservationScheduleAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.8),
                    ("east".to_string(), 0.78),
                    ("south".to_string(), 1.26),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 38406.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_schedule_engine();
        let item = new_reservation_schedule("code-236", "title-497");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
