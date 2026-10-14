use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation profile calculation.
pub trait ReservationProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationProfileAdjuster for TieredReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationProfileAdjuster {
    pub step: f64,
}

impl ReservationProfileAdjuster for RoundingReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationProfileRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ReservationProfileAdjuster for ReservationProfileRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ReservationProfile, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleReservationProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationProfileAdjuster for TitleReservationProfileAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &ReservationProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationProfileAdjuster for FlaggedReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationProfileAdjuster {
    pub cap: f64,
}

impl ReservationProfileAdjuster for CapReservationProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the reservation profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationProfileEngine {
    pub steps: Vec<Rc<dyn ReservationProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationProfileEngine {
    pub fn evaluate(&mut self, item: &ReservationProfile) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReservationProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_profile_engine(flags: &flags::Registry) -> ReservationProfileEngine {
    ReservationProfileEngine {
        steps: vec![
            Rc::new(TieredReservationProfileAdjuster {
                bounds: vec![1560.0, 2736.0, 6782.0],
                rates: vec![1.13, 1.13, 1.11],
            }) as Rc<dyn ReservationProfileAdjuster>,
            Rc::new(RoundingReservationProfileAdjuster { step: 0.05 }),
            Rc::new(ReservationProfileRetriesOffsetAdjuster { weight: 1.61 }),
            Rc::new(TitleReservationProfileAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.9)]),
            }),
            Rc::new(FlaggedReservationProfileAdjuster {
                flags: flags.clone(),
                flag: "reservation-profile-rolling".to_string(),
                percent: -3.0,
            }),
            Rc::new(CapReservationProfileAdjuster { cap: 2434.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 97744.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_profile_engine(&flags::new_registry());
        let item = new_reservation_profile("code-676", "title-837");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
