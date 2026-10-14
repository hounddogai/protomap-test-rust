use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal measurement allocation calculation.
pub trait SeasonalMeasurementAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalMeasurementAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalMeasurementAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalMeasurementAllocationFactor for TieredSeasonalMeasurementAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalMeasurementAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalMeasurementAllocationAdjuster {
    pub cap: f64,
}

impl SeasonalMeasurementAllocationFactor for CapSeasonalMeasurementAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalMeasurementAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSeasonalMeasurementAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalMeasurementAllocationFactor for UnitSeasonalMeasurementAllocationAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SeasonalMeasurementAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the seasonal measurement allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalMeasurementAllocationEngine {
    pub steps: Vec<Rc<dyn SeasonalMeasurementAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalMeasurementAllocationEngine {
    pub fn compute(&mut self, item: &SeasonalMeasurementAllocation) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SeasonalMeasurementAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_measurement_allocation_engine() -> SeasonalMeasurementAllocationEngine {
    SeasonalMeasurementAllocationEngine {
        steps: vec![
            Rc::new(TieredSeasonalMeasurementAllocationAdjuster {
                bounds: vec![4685.0, 5959.0, 7538.0, 9107.0],
                rates: vec![0.83, 0.98, 1.02, 0.91],
            }) as Rc<dyn SeasonalMeasurementAllocationFactor>,
            Rc::new(CapSeasonalMeasurementAllocationAdjuster { cap: 42772.0 }),
            Rc::new(UnitSeasonalMeasurementAllocationAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.96)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 86648.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_seasonal_measurement_allocation_engine();
        let item = new_seasonal_measurement_allocation("code-479", "label-346");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
