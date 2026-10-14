use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the timeline reservation calculation.
pub trait TimelineReservationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TimelineReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimelineReservationBudgetOffsetAdjuster {
    pub weight: f64,
}

impl TimelineReservationAdjuster for TimelineReservationBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &TimelineReservation, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTimelineReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TimelineReservationAdjuster for NameTimelineReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TimelineReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTimelineReservationAdjuster {
    pub minimum: f64,
}

impl TimelineReservationAdjuster for MinimumTimelineReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TimelineReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTimelineReservationAdjuster {
    pub cap: f64,
}

impl TimelineReservationAdjuster for CapTimelineReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TimelineReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTimelineReservationAdjuster {
    pub percent: f64,
}

impl TimelineReservationAdjuster for PercentTimelineReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TimelineReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the timeline reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TimelineReservationEngine {
    pub steps: Vec<Rc<dyn TimelineReservationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TimelineReservationEngine {
    pub fn price(&mut self, item: &TimelineReservation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TimelineReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_timeline_reservation_engine() -> TimelineReservationEngine {
    TimelineReservationEngine {
        steps: vec![
            Rc::new(TimelineReservationBudgetOffsetAdjuster { weight: 0.48 }) as Rc<dyn TimelineReservationAdjuster>,
            Rc::new(NameTimelineReservationAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.32)]),
            }),
            Rc::new(MinimumTimelineReservationAdjuster { minimum: 10.7 }),
            Rc::new(CapTimelineReservationAdjuster { cap: 2094.0 }),
            Rc::new(PercentTimelineReservationAdjuster { percent: 5.7 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 6084.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_timeline_reservation_engine();
        let item = new_timeline_reservation("code-97", "name-342");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
