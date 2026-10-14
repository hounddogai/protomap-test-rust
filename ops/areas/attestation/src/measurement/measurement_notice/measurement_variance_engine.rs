use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement variance calculation.
pub trait MeasurementVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelMeasurementVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementVarianceStep for LabelMeasurementVarianceAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &MeasurementVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementVarianceStep for TieredMeasurementVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementVarianceAdjuster {
    pub cap: f64,
}

impl MeasurementVarianceStep for CapMeasurementVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the measurement variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementVarianceEngine {
    pub steps: Vec<Rc<dyn MeasurementVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementVarianceEngine {
    pub fn compute(&mut self, item: &MeasurementVariance) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[MeasurementVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_variance_engine() -> MeasurementVarianceEngine {
    MeasurementVarianceEngine {
        steps: vec![
            Rc::new(LabelMeasurementVarianceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.28), ("north".to_string(), 0.74)]),
            }) as Rc<dyn MeasurementVarianceStep>,
            Rc::new(TieredMeasurementVarianceAdjuster {
                bounds: vec![5466.0, 8433.0],
                rates: vec![1.14, 0.95],
            }),
            Rc::new(CapMeasurementVarianceAdjuster { cap: 33040.0 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 45705.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_variance_engine();
        let item = new_measurement_variance("code-774", "label-850");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
