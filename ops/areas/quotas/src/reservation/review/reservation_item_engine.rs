use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation item calculation.
pub trait ReservationItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationItemAdjuster {
    pub step: f64,
}

impl ReservationItemAdjuster for RoundingReservationItemAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationItem, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationItemAdjuster for FlaggedReservationItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleReservationItemAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationItemAdjuster for TitleReservationItemAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ReservationItem, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationItemUpdatedAtOffsetAdjuster {
    pub weight: f64,
}

impl ReservationItemAdjuster for ReservationItemUpdatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus updated at times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationItem, value: f64) -> f64 {
        value + (item.updated_at as f64) * self.weight
    }
}

/// Runs the reservation item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationItemEngine {
    pub steps: Vec<Rc<dyn ReservationItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationItemEngine {
    pub fn run(&mut self, item: &ReservationItem) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[ReservationItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_reservation_item_engine(flags: &flags::Registry) -> ReservationItemEngine {
    ReservationItemEngine {
        steps: vec![
            Rc::new(RoundingReservationItemAdjuster { step: 5.0 }) as Rc<dyn ReservationItemAdjuster>,
            Rc::new(FlaggedReservationItemAdjuster {
                flags: flags.clone(),
                flag: "reservation-item-fallback".to_string(),
                percent: 2.2,
            }),
            Rc::new(TitleReservationItemAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.08), ("central".to_string(), 0.75)]),
            }),
            Rc::new(ReservationItemUpdatedAtOffsetAdjuster { weight: 0.33 }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 62036.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_item_engine(&flags::new_registry());
        let item = new_reservation_item("code-120", "title-591");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
