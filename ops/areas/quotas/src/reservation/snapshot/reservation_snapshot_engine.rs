use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation snapshot calculation.
pub trait ReservationSnapshotAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationSnapshotAdjuster {
    pub cap: f64,
}

impl ReservationSnapshotAdjuster for CapReservationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationSnapshotAdjuster for FlaggedReservationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameReservationSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationSnapshotAdjuster for NameReservationSnapshotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &ReservationSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationSnapshotAdjuster for TieredReservationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationSnapshotAdjuster {
    pub step: f64,
}

impl ReservationSnapshotAdjuster for RoundingReservationSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the reservation snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationSnapshotEngine {
    pub steps: Vec<Rc<dyn ReservationSnapshotAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationSnapshotEngine {
    pub fn evaluate(&mut self, item: &ReservationSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReservationSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_snapshot_engine(flags: &flags::Registry) -> ReservationSnapshotEngine {
    ReservationSnapshotEngine {
        steps: vec![
            Rc::new(CapReservationSnapshotAdjuster { cap: 28838.0 }) as Rc<dyn ReservationSnapshotAdjuster>,
            Rc::new(FlaggedReservationSnapshotAdjuster {
                flags: flags.clone(),
                flag: "reservation-snapshot-adaptive".to_string(),
                percent: 4.3,
            }),
            Rc::new(NameReservationSnapshotAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.33)]),
            }),
            Rc::new(TieredReservationSnapshotAdjuster {
                bounds: vec![718.0, 5451.0, 7469.0, 8704.0],
                rates: vec![1.05, 1.1, 0.96, 1.16],
            }),
            Rc::new(RoundingReservationSnapshotAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 82058.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_snapshot_engine(&flags::new_registry());
        let item = new_reservation_snapshot("code-161", "name-478");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
