use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive region pin metric calculation.
pub trait AdaptiveRegionPinMetricStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRegionPinMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRegionPinMetricAdjuster {
    pub step: f64,
}

impl AdaptiveRegionPinMetricStep for RoundingAdaptiveRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRegionPinMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinMetricMaximumOffsetAdjuster {
    pub weight: f64,
}

impl AdaptiveRegionPinMetricStep for AdaptiveRegionPinMetricMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &AdaptiveRegionPinMetric, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptiveRegionPinMetricAdjuster {
    pub minimum: f64,
}

impl AdaptiveRegionPinMetricStep for MinimumAdaptiveRegionPinMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptiveRegionPinMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the adaptive region pin metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRegionPinMetricEngine {
    pub steps: Vec<Rc<dyn AdaptiveRegionPinMetricStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRegionPinMetricEngine {
    pub fn price(&mut self, item: &AdaptiveRegionPinMetric) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRegionPinMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_region_pin_metric_engine() -> AdaptiveRegionPinMetricEngine {
    AdaptiveRegionPinMetricEngine {
        steps: vec![
            Rc::new(RoundingAdaptiveRegionPinMetricAdjuster { step: 0.1 }) as Rc<dyn AdaptiveRegionPinMetricStep>,
            Rc::new(AdaptiveRegionPinMetricMaximumOffsetAdjuster { weight: 1.66 }),
            Rc::new(MinimumAdaptiveRegionPinMetricAdjuster { minimum: 28.8 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 39379.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_region_pin_metric_engine();
        let item = new_adaptive_region_pin_metric("code-981", "title-743");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
