use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending canary reservation calculation.
pub trait PendingCanaryReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingCanaryReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPendingCanaryReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingCanaryReservationModifier for ChannelPendingCanaryReservationAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PendingCanaryReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingCanaryReservationAdjuster {
    pub step: f64,
}

impl PendingCanaryReservationModifier for RoundingPendingCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingCanaryReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingCanaryReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingCanaryReservationModifier for FlaggedPendingCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingCanaryReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingCanaryReservationAdjuster {
    pub cap: f64,
}

impl PendingCanaryReservationModifier for CapPendingCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingCanaryReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingCanaryReservationAdjuster {
    pub percent: f64,
}

impl PendingCanaryReservationModifier for PercentPendingCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingCanaryReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingCanaryReservationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingCanaryReservationModifier for TieredPendingCanaryReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingCanaryReservation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending canary reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingCanaryReservationEngine {
    pub steps: Vec<Rc<dyn PendingCanaryReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingCanaryReservationEngine {
    pub fn evaluate(&mut self, item: &PendingCanaryReservation) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingCanaryReservationModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingCanaryReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_canary_reservation_engine(flags: &flags::Registry) -> PendingCanaryReservationEngine {
    PendingCanaryReservationEngine {
        steps: vec![
            Rc::new(ChannelPendingCanaryReservationAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.85)]),
            }) as Rc<dyn PendingCanaryReservationModifier>,
            Rc::new(RoundingPendingCanaryReservationAdjuster { step: 0.25 }),
            Rc::new(FlaggedPendingCanaryReservationAdjuster {
                flags: flags.clone(),
                flag: "pending-canary-reservation-lenient".to_string(),
                percent: 8.3,
            }),
            Rc::new(CapPendingCanaryReservationAdjuster { cap: 36839.0 }),
            Rc::new(PercentPendingCanaryReservationAdjuster { percent: 7.7 }),
            Rc::new(TieredPendingCanaryReservationAdjuster {
                bounds: vec![884.0, 2662.0, 4526.0, 7290.0],
                rates: vec![1.13, 1.09, 0.86, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 68644.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_canary_reservation_engine(&flags::new_registry());
        let item = new_pending_canary_reservation("code-626", "label-532");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
