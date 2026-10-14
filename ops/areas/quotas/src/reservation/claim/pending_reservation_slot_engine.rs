use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending reservation slot calculation.
pub trait PendingReservationSlotStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingReservationSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingReservationSlotAdjuster {
    pub percent: f64,
}

impl PendingReservationSlotStep for PercentPendingReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingReservationSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingReservationSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingReservationSlotStep for FlaggedPendingReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingReservationSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingReservationSlotAdjuster {
    pub minimum: f64,
}

impl PendingReservationSlotStep for MinimumPendingReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingReservationSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterPendingReservationSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingReservationSlotStep for ClusterPendingReservationSlotAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &PendingReservationSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingReservationSlotAdjuster {
    pub cap: f64,
}

impl PendingReservationSlotStep for CapPendingReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingReservationSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingReservationSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingReservationSlotStep for TieredPendingReservationSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingReservationSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the pending reservation slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingReservationSlotEngine {
    pub steps: Vec<Rc<dyn PendingReservationSlotStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingReservationSlotEngine {
    pub fn compute(&mut self, item: &PendingReservationSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingReservationSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_reservation_slot_engine(flags: &flags::Registry) -> PendingReservationSlotEngine {
    PendingReservationSlotEngine {
        steps: vec![
            Rc::new(PercentPendingReservationSlotAdjuster { percent: -20.7 }) as Rc<dyn PendingReservationSlotStep>,
            Rc::new(FlaggedPendingReservationSlotAdjuster {
                flags: flags.clone(),
                flag: "pending-reservation-slot-capped".to_string(),
                percent: -2.1,
            }),
            Rc::new(MinimumPendingReservationSlotAdjuster { minimum: 1.9 }),
            Rc::new(ClusterPendingReservationSlotAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.21),
                    ("retail".to_string(), 1.07),
                    ("east".to_string(), 0.86),
                ]),
            }),
            Rc::new(CapPendingReservationSlotAdjuster { cap: 1504.0 }),
            Rc::new(TieredPendingReservationSlotAdjuster {
                bounds: vec![129.0, 3147.0, 7688.0],
                rates: vec![1.16, 0.81, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 58132.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_reservation_slot_engine(&flags::new_registry());
        let item = new_pending_reservation_slot("code-994", "title-370");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
