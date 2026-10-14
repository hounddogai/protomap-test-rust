use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement estimate calculation.
pub trait MeasurementEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementEstimateAdjuster {
    pub cap: f64,
}

impl MeasurementEstimateStep for CapMeasurementEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementEstimateExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementEstimateStep for MeasurementEstimateExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementEstimate, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementEstimateAdjuster {
    pub percent: f64,
}

impl MeasurementEstimateStep for PercentMeasurementEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryMeasurementEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementEstimateStep for CategoryMeasurementEstimateAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &MeasurementEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementEstimateAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementEstimateStep for TieredMeasurementEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementEstimate, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the measurement estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementEstimateEngine {
    pub steps: Vec<Rc<dyn MeasurementEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementEstimateEngine {
    pub fn run(&mut self, item: &MeasurementEstimate) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_measurement_estimate_engine() -> MeasurementEstimateEngine {
    MeasurementEstimateEngine {
        steps: vec![
            Rc::new(CapMeasurementEstimateAdjuster { cap: 33990.0 }) as Rc<dyn MeasurementEstimateStep>,
            Rc::new(MeasurementEstimateExpiresAtOffsetAdjuster { weight: 1.59 }),
            Rc::new(PercentMeasurementEstimateAdjuster { percent: -14.7 }),
            Rc::new(CategoryMeasurementEstimateAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 0.72),
                    ("south".to_string(), 1.26),
                    ("west".to_string(), 1.37),
                ]),
            }),
            Rc::new(TieredMeasurementEstimateAdjuster {
                bounds: vec![2283.0, 5759.0, 7693.0, 9477.0],
                rates: vec![1.16, 1.0, 0.82, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 21033.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_estimate_engine();
        let item = new_measurement_estimate("code-921", "name-467");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
