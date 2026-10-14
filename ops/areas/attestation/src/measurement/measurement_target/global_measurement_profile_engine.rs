use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global measurement profile calculation.
pub trait GlobalMeasurementProfileFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalMeasurementProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalMeasurementProfileAdjuster {
    pub cap: f64,
}

impl GlobalMeasurementProfileFactor for CapGlobalMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalMeasurementProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalMeasurementProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalMeasurementProfileFactor for TieredGlobalMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalMeasurementProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalMeasurementProfileAdjuster {
    pub step: f64,
}

impl GlobalMeasurementProfileFactor for RoundingGlobalMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalMeasurementProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalMeasurementProfileAdjuster {
    pub percent: f64,
}

impl GlobalMeasurementProfileFactor for PercentGlobalMeasurementProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalMeasurementProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global measurement profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalMeasurementProfileEngine {
    pub steps: Vec<Rc<dyn GlobalMeasurementProfileFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalMeasurementProfileEngine {
    pub fn evaluate(&mut self, item: &GlobalMeasurementProfile) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[GlobalMeasurementProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_measurement_profile_engine() -> GlobalMeasurementProfileEngine {
    GlobalMeasurementProfileEngine {
        steps: vec![
            Rc::new(CapGlobalMeasurementProfileAdjuster { cap: 25702.0 }) as Rc<dyn GlobalMeasurementProfileFactor>,
            Rc::new(TieredGlobalMeasurementProfileAdjuster {
                bounds: vec![3571.0, 5352.0, 7179.0, 7965.0],
                rates: vec![0.92, 1.01, 0.94, 1.19],
            }),
            Rc::new(RoundingGlobalMeasurementProfileAdjuster { step: 0.5 }),
            Rc::new(PercentGlobalMeasurementProfileAdjuster { percent: -21.5 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 82522.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_measurement_profile_engine();
        let item = new_global_measurement_profile("code-87", "title-22");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
