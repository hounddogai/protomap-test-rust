use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement snapshot calculation.
pub trait MeasurementSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementSnapshotAdjuster {
    pub minimum: f64,
}

impl MeasurementSnapshotModifier for MinimumMeasurementSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementSnapshotAdjuster {
    pub percent: f64,
}

impl MeasurementSnapshotModifier for PercentMeasurementSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementSnapshot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementSnapshotModifier for TieredMeasurementSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementSnapshotAdjuster {
    pub cap: f64,
}

impl MeasurementSnapshotModifier for CapMeasurementSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMeasurementSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementSnapshotModifier for SegmentMeasurementSnapshotAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &MeasurementSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementSnapshotEngine {
    pub steps: Vec<Rc<dyn MeasurementSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementSnapshotEngine {
    pub fn evaluate(&mut self, item: &MeasurementSnapshot) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_measurement_snapshot_engine() -> MeasurementSnapshotEngine {
    MeasurementSnapshotEngine {
        steps: vec![
            Rc::new(MinimumMeasurementSnapshotAdjuster { minimum: 34.1 }) as Rc<dyn MeasurementSnapshotModifier>,
            Rc::new(PercentMeasurementSnapshotAdjuster { percent: 1.8 }),
            Rc::new(TieredMeasurementSnapshotAdjuster {
                bounds: vec![92.0, 4497.0],
                rates: vec![0.94, 0.97],
            }),
            Rc::new(CapMeasurementSnapshotAdjuster { cap: 18966.0 }),
            Rc::new(SegmentMeasurementSnapshotAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.84),
                    ("central".to_string(), 0.71),
                    ("west".to_string(), 0.98),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 18589.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_snapshot_engine();
        let item = new_measurement_snapshot("code-680", "title-749");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
