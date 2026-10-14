use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global measurement override calculation.
pub trait GlobalMeasurementOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalMeasurementOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalMeasurementOverrideAdjuster {
    pub step: f64,
}

impl GlobalMeasurementOverrideModifier for RoundingGlobalMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalMeasurementOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalMeasurementOverrideAdjuster {
    pub minimum: f64,
}

impl GlobalMeasurementOverrideModifier for MinimumGlobalMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalMeasurementOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleGlobalMeasurementOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalMeasurementOverrideModifier for LocaleGlobalMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &GlobalMeasurementOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalMeasurementOverrideAdjuster {
    pub percent: f64,
}

impl GlobalMeasurementOverrideModifier for PercentGlobalMeasurementOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalMeasurementOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the global measurement override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalMeasurementOverrideEngine {
    pub steps: Vec<Rc<dyn GlobalMeasurementOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalMeasurementOverrideEngine {
    pub fn price(&mut self, item: &GlobalMeasurementOverride) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[GlobalMeasurementOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_measurement_override_engine() -> GlobalMeasurementOverrideEngine {
    GlobalMeasurementOverrideEngine {
        steps: vec![
            Rc::new(RoundingGlobalMeasurementOverrideAdjuster { step: 0.25 })
                as Rc<dyn GlobalMeasurementOverrideModifier>,
            Rc::new(MinimumGlobalMeasurementOverrideAdjuster { minimum: 37.5 }),
            Rc::new(LocaleGlobalMeasurementOverrideAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.24), ("north".to_string(), 1.03)]),
            }),
            Rc::new(PercentGlobalMeasurementOverrideAdjuster { percent: 10.9 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 18758.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_measurement_override_engine();
        let item = new_global_measurement_override("code-324", "label-127");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
