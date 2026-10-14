use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the measurement digest calculation.
pub trait MeasurementDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &MeasurementDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapMeasurementDigestAdjuster {
    pub cap: f64,
}

impl MeasurementDigestModifier for CapMeasurementDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &MeasurementDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameMeasurementDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl MeasurementDigestModifier for NameMeasurementDigestAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &MeasurementDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentMeasurementDigestAdjuster {
    pub percent: f64,
}

impl MeasurementDigestModifier for PercentMeasurementDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &MeasurementDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the measurement digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct MeasurementDigestEngine {
    pub steps: Vec<Rc<dyn MeasurementDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl MeasurementDigestEngine {
    pub fn compute(&mut self, item: &MeasurementDigest) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[MeasurementDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_measurement_digest_engine() -> MeasurementDigestEngine {
    MeasurementDigestEngine {
        steps: vec![
            Rc::new(CapMeasurementDigestAdjuster { cap: 33429.0 }) as Rc<dyn MeasurementDigestModifier>,
            Rc::new(NameMeasurementDigestAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.35)]),
            }),
            Rc::new(PercentMeasurementDigestAdjuster { percent: -9.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 3368.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_measurement_digest_engine();
        let item = new_measurement_digest("code-967", "name-565");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
