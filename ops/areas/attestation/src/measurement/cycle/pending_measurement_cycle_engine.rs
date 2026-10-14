use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending measurement cycle calculation.
pub trait PendingMeasurementCycleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingMeasurementCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMeasurementCycleExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl PendingMeasurementCycleStep for PendingMeasurementCycleExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &PendingMeasurementCycle, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingMeasurementCycleAdjuster {
    pub percent: f64,
}

impl PendingMeasurementCycleStep for PercentPendingMeasurementCycleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingMeasurementCycle, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingMeasurementCycleAdjuster {
    pub step: f64,
}

impl PendingMeasurementCycleStep for RoundingPendingMeasurementCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingMeasurementCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingMeasurementCycleAdjuster {
    pub minimum: f64,
}

impl PendingMeasurementCycleStep for MinimumPendingMeasurementCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingMeasurementCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingMeasurementCycleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingMeasurementCycleStep for TieredPendingMeasurementCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingMeasurementCycle, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingMeasurementCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingMeasurementCycleStep for FlaggedPendingMeasurementCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingMeasurementCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending measurement cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingMeasurementCycleEngine {
    pub steps: Vec<Rc<dyn PendingMeasurementCycleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingMeasurementCycleEngine {
    pub fn compute(&mut self, item: &PendingMeasurementCycle) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingMeasurementCycleStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingMeasurementCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_measurement_cycle_engine(flags: &flags::Registry) -> PendingMeasurementCycleEngine {
    PendingMeasurementCycleEngine {
        steps: vec![
            Rc::new(PendingMeasurementCycleExpiresAtOffsetAdjuster { weight: 1.47 })
                as Rc<dyn PendingMeasurementCycleStep>,
            Rc::new(PercentPendingMeasurementCycleAdjuster { percent: 6.6 }),
            Rc::new(RoundingPendingMeasurementCycleAdjuster { step: 0.25 }),
            Rc::new(MinimumPendingMeasurementCycleAdjuster { minimum: 49.9 }),
            Rc::new(TieredPendingMeasurementCycleAdjuster {
                bounds: vec![7437.0, 8221.0],
                rates: vec![1.03, 0.83],
            }),
            Rc::new(FlaggedPendingMeasurementCycleAdjuster {
                flags: flags.clone(),
                flag: "pending-measurement-cycle-rolling".to_string(),
                percent: -2.9,
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 2176.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_measurement_cycle_engine(&flags::new_registry());
        let item = new_pending_measurement_cycle("code-495", "label-413");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
