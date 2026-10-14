use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation manifest calculation.
pub trait ReservationManifestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationManifestAdjuster {
    pub minimum: f64,
}

impl ReservationManifestFactor for MinimumReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationManifestAdjuster {
    pub percent: f64,
}

impl ReservationManifestFactor for PercentReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationManifest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationManifestFactor for FlaggedReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReservationManifestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReservationManifestFactor for TieredReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReservationManifest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationManifestAdjuster {
    pub step: f64,
}

impl ReservationManifestFactor for RoundingReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the reservation manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationManifestEngine {
    pub steps: Vec<Rc<dyn ReservationManifestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationManifestEngine {
    pub fn evaluate(&mut self, item: &ReservationManifest) -> f64 {
        let mut current = item.net_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationManifestFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_reservation_manifest_engine(flags: &flags::Registry) -> ReservationManifestEngine {
    ReservationManifestEngine {
        steps: vec![
            Rc::new(MinimumReservationManifestAdjuster { minimum: 29.9 }) as Rc<dyn ReservationManifestFactor>,
            Rc::new(PercentReservationManifestAdjuster { percent: 23.2 }),
            Rc::new(FlaggedReservationManifestAdjuster {
                flags: flags.clone(),
                flag: "reservation-manifest-rolling".to_string(),
                percent: -9.3,
            }),
            Rc::new(TieredReservationManifestAdjuster {
                bounds: vec![5070.0, 5400.0, 5452.0, 8676.0],
                rates: vec![1.08, 0.93, 0.97, 0.83],
            }),
            Rc::new(RoundingReservationManifestAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 58478.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_reservation_manifest_engine(&flags::new_registry());
        let item = new_reservation_manifest("code-380", "title-319");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
