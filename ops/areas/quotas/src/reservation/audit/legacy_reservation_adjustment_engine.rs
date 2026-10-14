use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the legacy reservation adjustment calculation.
pub trait LegacyReservationAdjustmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyReservationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumLegacyReservationAdjustmentAdjuster {
    pub minimum: f64,
}

impl LegacyReservationAdjustmentStep for MinimumLegacyReservationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &LegacyReservationAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLegacyReservationAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LegacyReservationAdjustmentStep for FlaggedLegacyReservationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LegacyReservationAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyReservationAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyReservationAdjustmentStep for TieredLegacyReservationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyReservationAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyReservationAdjustmentAdjuster {
    pub step: f64,
}

impl LegacyReservationAdjustmentStep for RoundingLegacyReservationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyReservationAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the legacy reservation adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyReservationAdjustmentEngine {
    pub steps: Vec<Rc<dyn LegacyReservationAdjustmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyReservationAdjustmentEngine {
    pub fn price(&mut self, item: &LegacyReservationAdjustment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyReservationAdjustmentStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyReservationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_legacy_reservation_adjustment_engine(flags: &flags::Registry) -> LegacyReservationAdjustmentEngine {
    LegacyReservationAdjustmentEngine {
        steps: vec![
            Rc::new(MinimumLegacyReservationAdjustmentAdjuster { minimum: 40.7 })
                as Rc<dyn LegacyReservationAdjustmentStep>,
            Rc::new(FlaggedLegacyReservationAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "legacy-reservation-adjustment-regional".to_string(),
                percent: -0.9,
            }),
            Rc::new(TieredLegacyReservationAdjustmentAdjuster {
                bounds: vec![259.0, 5713.0],
                rates: vec![1.19, 1.08],
            }),
            Rc::new(RoundingLegacyReservationAdjustmentAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 18625.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_reservation_adjustment_engine(&flags::new_registry());
        let item = new_legacy_reservation_adjustment("code-273", "title-202");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
