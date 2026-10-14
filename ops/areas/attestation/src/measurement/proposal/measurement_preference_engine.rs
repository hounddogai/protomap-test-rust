use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement preference calculation.
pub trait MeasurementPreferenceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementPreferenceAdjuster {
    pub step: f64,
}

impl MeasurementPreferenceStep for RoundingMeasurementPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementPreferenceAdjuster {
    pub cap: f64,
}

impl MeasurementPreferenceStep for CapMeasurementPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementPreference, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementPreferenceStep for TieredMeasurementPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementPreferenceAdjuster {
    pub minimum: f64,
}

impl MeasurementPreferenceStep for MinimumMeasurementPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementPreferenceAdjuster {
    pub percent: f64,
}

impl MeasurementPreferenceStep for PercentMeasurementPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the measurement preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementPreferenceEngine {
    pub steps: Vec<Rc<dyn MeasurementPreferenceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementPreferenceEngine {
    pub fn price(&mut self, item: &MeasurementPreference) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementPreferenceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_measurement_preference_engine() -> MeasurementPreferenceEngine {
    MeasurementPreferenceEngine {
        steps: vec![
            Rc::new(RoundingMeasurementPreferenceAdjuster { step: 1.0 }) as Rc<dyn MeasurementPreferenceStep>,
            Rc::new(CapMeasurementPreferenceAdjuster { cap: 12901.0 }),
            Rc::new(TieredMeasurementPreferenceAdjuster {
                bounds: vec![2503.0, 5225.0, 8102.0, 8846.0],
                rates: vec![0.97, 0.88, 1.15, 1.12],
            }),
            Rc::new(MinimumMeasurementPreferenceAdjuster { minimum: 19.7 }),
            Rc::new(PercentMeasurementPreferenceAdjuster { percent: 16.1 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 53100.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_preference_engine();
        let item = new_measurement_preference("code-378", "title-222");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
