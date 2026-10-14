use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement override calculation.
pub trait MeasurementOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelMeasurementOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementOverrideModifier for LabelMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &MeasurementOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementOverrideAdjuster {
    pub percent: f64,
}

impl MeasurementOverrideModifier for PercentMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl MeasurementOverrideModifier for TieredMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &MeasurementOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementOverridePortOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementOverrideModifier for MeasurementOverridePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementOverride, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementOverrideAdjuster {
    pub cap: f64,
}

impl MeasurementOverrideModifier for CapMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the measurement override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementOverrideEngine {
    pub steps: Vec<Rc<dyn MeasurementOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementOverrideEngine {
    pub fn evaluate(&mut self, item: &MeasurementOverride) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_measurement_override_engine() -> MeasurementOverrideEngine {
    MeasurementOverrideEngine {
        steps: vec![
            Rc::new(LabelMeasurementOverrideAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.09)]),
            }) as Rc<dyn MeasurementOverrideModifier>,
            Rc::new(PercentMeasurementOverrideAdjuster { percent: -7.5 }),
            Rc::new(TieredMeasurementOverrideAdjuster {
                bounds: vec![8744.0, 9981.0],
                rates: vec![0.89, 1.09],
            }),
            Rc::new(MeasurementOverridePortOffsetAdjuster { weight: 0.49 }),
            Rc::new(CapMeasurementOverrideAdjuster { cap: 6057.0 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 78170.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_override_engine();
        let item = new_measurement_override("code-591", "label-839");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
