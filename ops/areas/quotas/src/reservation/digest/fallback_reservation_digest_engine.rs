use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback reservation digest calculation.
pub trait FallbackReservationDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackReservationDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackReservationDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackReservationDigestAdjuster for TieredFallbackReservationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackReservationDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackReservationDigestAdjuster {
    pub cap: f64,
}

impl FallbackReservationDigestAdjuster for CapFallbackReservationDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackReservationDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackReservationDigestAdjuster {
    pub step: f64,
}

impl FallbackReservationDigestAdjuster for RoundingFallbackReservationDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackReservationDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackReservationDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackReservationDigestAdjuster for FlaggedFallbackReservationDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackReservationDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationDigestTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl FallbackReservationDigestAdjuster for FallbackReservationDigestTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackReservationDigest, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the fallback reservation digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackReservationDigestEngine {
    pub steps: Vec<Rc<dyn FallbackReservationDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackReservationDigestEngine {
    pub fn evaluate(&mut self, item: &FallbackReservationDigest) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FallbackReservationDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_reservation_digest_engine(flags: &flags::Registry) -> FallbackReservationDigestEngine {
    FallbackReservationDigestEngine {
        steps: vec![
            Rc::new(TieredFallbackReservationDigestAdjuster {
                bounds: vec![495.0, 1165.0],
                rates: vec![0.91, 1.04],
            }) as Rc<dyn FallbackReservationDigestAdjuster>,
            Rc::new(CapFallbackReservationDigestAdjuster { cap: 26716.0 }),
            Rc::new(RoundingFallbackReservationDigestAdjuster { step: 0.5 }),
            Rc::new(FlaggedFallbackReservationDigestAdjuster {
                flags: flags.clone(),
                flag: "fallback-reservation-digest-scheduled".to_string(),
                percent: 1.8,
            }),
            Rc::new(FallbackReservationDigestTtlSecondsOffsetAdjuster { weight: 1.25 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 54748.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_reservation_digest_engine(&flags::new_registry());
        let item = new_fallback_reservation_digest("code-172", "name-448");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
