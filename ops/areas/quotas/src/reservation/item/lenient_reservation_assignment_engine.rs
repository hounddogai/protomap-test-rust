use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient reservation assignment calculation.
pub trait LenientReservationAssignmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientReservationAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientReservationAssignmentAdjuster {
    pub percent: f64,
}

impl LenientReservationAssignmentStep for PercentLenientReservationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientReservationAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLenientReservationAssignmentAdjuster {
    pub minimum: f64,
}

impl LenientReservationAssignmentStep for MinimumLenientReservationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LenientReservationAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientReservationAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientReservationAssignmentStep for FlaggedLenientReservationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientReservationAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientReservationAssignmentAdjuster {
    pub step: f64,
}

impl LenientReservationAssignmentStep for RoundingLenientReservationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientReservationAssignment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLenientReservationAssignmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LenientReservationAssignmentStep for TieredLenientReservationAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LenientReservationAssignment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the lenient reservation assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientReservationAssignmentEngine {
    pub steps: Vec<Rc<dyn LenientReservationAssignmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientReservationAssignmentEngine {
    pub fn run(&mut self, item: &LenientReservationAssignment) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LenientReservationAssignmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LenientReservationAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_lenient_reservation_assignment_engine(flags: &flags::Registry) -> LenientReservationAssignmentEngine {
    LenientReservationAssignmentEngine {
        steps: vec![
            Rc::new(PercentLenientReservationAssignmentAdjuster { percent: 11.9 })
                as Rc<dyn LenientReservationAssignmentStep>,
            Rc::new(MinimumLenientReservationAssignmentAdjuster { minimum: 22.2 }),
            Rc::new(FlaggedLenientReservationAssignmentAdjuster {
                flags: flags.clone(),
                flag: "lenient-reservation-assignment-manual".to_string(),
                percent: 7.7,
            }),
            Rc::new(RoundingLenientReservationAssignmentAdjuster { step: 0.5 }),
            Rc::new(TieredLenientReservationAssignmentAdjuster {
                bounds: vec![5547.0, 8350.0],
                rates: vec![1.15, 0.82],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 53194.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_lenient_reservation_assignment_engine(&flags::new_registry());
        let item = new_lenient_reservation_assignment("code-514", "name-946");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
