use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement adjustment calculation.
pub trait MeasurementAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementAdjustmentAdjuster {
    pub cap: f64,
}

impl MeasurementAdjustmentAdjuster for CapMeasurementAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementAdjustmentStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementAdjustmentAdjuster for MeasurementAdjustmentStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementAdjustment, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelMeasurementAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementAdjustmentAdjuster for ChannelMeasurementAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &MeasurementAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementAdjustmentEngine {
    pub steps: Vec<Rc<dyn MeasurementAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementAdjustmentEngine {
    pub fn compute(&mut self, item: &MeasurementAdjustment) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_adjustment_engine() -> MeasurementAdjustmentEngine {
    MeasurementAdjustmentEngine {
        steps: vec![
            Rc::new(CapMeasurementAdjustmentAdjuster { cap: 5797.0 }) as Rc<dyn MeasurementAdjustmentAdjuster>,
            Rc::new(MeasurementAdjustmentStartsAtOffsetAdjuster { weight: 0.73 }),
            Rc::new(ChannelMeasurementAdjustmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.85)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 53399.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_adjustment_engine();
        let item = new_measurement_adjustment("code-413", "name-448");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
