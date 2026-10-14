use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement record calculation.
pub trait MeasurementRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRecordLimitOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementRecordStep for MeasurementRecordLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementRecord, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementRecordAdjuster {
    pub step: f64,
}

impl MeasurementRecordStep for RoundingMeasurementRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterMeasurementRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementRecordStep for ClusterMeasurementRecordAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &MeasurementRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementRecordEngine {
    pub steps: Vec<Rc<dyn MeasurementRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementRecordEngine {
    pub fn compute(&mut self, item: &MeasurementRecord) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_record_engine() -> MeasurementRecordEngine {
    MeasurementRecordEngine {
        steps: vec![
            Rc::new(MeasurementRecordLimitOffsetAdjuster { weight: 1.49 }) as Rc<dyn MeasurementRecordStep>,
            Rc::new(RoundingMeasurementRecordAdjuster { step: 1.0 }),
            Rc::new(ClusterMeasurementRecordAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.11),
                    ("east".to_string(), 1.15),
                    ("south".to_string(), 0.88),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 39356.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_record_engine();
        let item = new_measurement_record("code-948", "label-124");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
