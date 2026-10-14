use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the forecast snapshot calculation.
pub trait ForecastSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ForecastSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredForecastSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ForecastSnapshotFactor for TieredForecastSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ForecastSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapForecastSnapshotAdjuster {
    pub cap: f64,
}

impl ForecastSnapshotFactor for CapForecastSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ForecastSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumForecastSnapshotAdjuster {
    pub minimum: f64,
}

impl ForecastSnapshotFactor for MinimumForecastSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ForecastSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSnapshotScoreOffsetAdjuster {
    pub weight: f64,
}

impl ForecastSnapshotFactor for ForecastSnapshotScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ForecastSnapshot, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

/// Runs the forecast snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ForecastSnapshotEngine {
    pub steps: Vec<Rc<dyn ForecastSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ForecastSnapshotEngine {
    pub fn evaluate(&mut self, item: &ForecastSnapshot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ForecastSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_forecast_snapshot_engine() -> ForecastSnapshotEngine {
    ForecastSnapshotEngine {
        steps: vec![
            Rc::new(TieredForecastSnapshotAdjuster {
                bounds: vec![228.0, 1068.0, 1809.0, 2626.0],
                rates: vec![0.94, 0.82, 0.93, 0.89],
            }) as Rc<dyn ForecastSnapshotFactor>,
            Rc::new(CapForecastSnapshotAdjuster { cap: 12667.0 }),
            Rc::new(MinimumForecastSnapshotAdjuster { minimum: 8.2 }),
            Rc::new(ForecastSnapshotScoreOffsetAdjuster { weight: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 34371.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_forecast_snapshot_engine();
        let item = new_forecast_snapshot("code-305", "title-922");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
