use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement exception calculation.
pub trait MeasurementExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementExceptionScoreOffsetAdjuster {
    pub weight: f64,
}

impl MeasurementExceptionModifier for MeasurementExceptionScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &MeasurementException, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementExceptionAdjuster {
    pub cap: f64,
}

impl MeasurementExceptionModifier for CapMeasurementExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingMeasurementExceptionAdjuster {
    pub step: f64,
}

impl MeasurementExceptionModifier for RoundingMeasurementExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &MeasurementException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumMeasurementExceptionAdjuster {
    pub minimum: f64,
}

impl MeasurementExceptionModifier for MinimumMeasurementExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &MeasurementException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceMeasurementExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementExceptionModifier for NamespaceMeasurementExceptionAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &MeasurementException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the measurement exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementExceptionEngine {
    pub steps: Vec<Rc<dyn MeasurementExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementExceptionEngine {
    pub fn compute(&mut self, item: &MeasurementException) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn MeasurementExceptionModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[MeasurementException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_exception_engine() -> MeasurementExceptionEngine {
    MeasurementExceptionEngine {
        steps: vec![
            Rc::new(MeasurementExceptionScoreOffsetAdjuster { weight: 0.5 }) as Rc<dyn MeasurementExceptionModifier>,
            Rc::new(CapMeasurementExceptionAdjuster { cap: 47367.0 }),
            Rc::new(RoundingMeasurementExceptionAdjuster { step: 0.1 }),
            Rc::new(MinimumMeasurementExceptionAdjuster { minimum: 29.3 }),
            Rc::new(NamespaceMeasurementExceptionAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.82), ("retail".to_string(), 0.97)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.6,
        ceiling: 23282.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_exception_engine();
        let item = new_measurement_exception("code-517", "name-957");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
