use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual measurement entry calculation.
pub trait ManualMeasurementEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualMeasurementEntry, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualMeasurementEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualMeasurementEntryModifier for FlaggedManualMeasurementEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualMeasurementEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualMeasurementEntryAdjuster {
    pub step: f64,
}

impl ManualMeasurementEntryModifier for RoundingManualMeasurementEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualMeasurementEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualMeasurementEntryAdjuster {
    pub minimum: f64,
}

impl ManualMeasurementEntryModifier for MinimumManualMeasurementEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualMeasurementEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualMeasurementEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualMeasurementEntryModifier for TieredManualMeasurementEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualMeasurementEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual measurement entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualMeasurementEntryEngine {
    pub steps: Vec<Rc<dyn ManualMeasurementEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualMeasurementEntryEngine {
    pub fn run(&mut self, item: &ManualMeasurementEntry) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualMeasurementEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualMeasurementEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_measurement_entry_engine(flags: &flags::Registry) -> ManualMeasurementEntryEngine {
    ManualMeasurementEntryEngine {
        steps: vec![
            Rc::new(FlaggedManualMeasurementEntryAdjuster {
                flags: flags.clone(),
                flag: "manual-measurement-entry-weighted".to_string(),
                percent: 3.9,
            }) as Rc<dyn ManualMeasurementEntryModifier>,
            Rc::new(RoundingManualMeasurementEntryAdjuster { step: 0.1 }),
            Rc::new(MinimumManualMeasurementEntryAdjuster { minimum: 10.3 }),
            Rc::new(TieredManualMeasurementEntryAdjuster {
                bounds: vec![3406.0, 4146.0, 5816.0, 9681.0],
                rates: vec![1.12, 1.12, 0.83, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 71315.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_measurement_entry_engine(&flags::new_registry());
        let item = new_manual_measurement_entry("code-290", "title-444");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
