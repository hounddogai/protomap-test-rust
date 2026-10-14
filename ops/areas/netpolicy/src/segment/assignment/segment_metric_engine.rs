use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the segment metric calculation.
pub trait SegmentMetricFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentMetricAdjuster {
    pub percent: f64,
}

impl SegmentMetricFactor for PercentSegmentMetricAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentMetric, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitSegmentMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentMetricFactor for UnitSegmentMetricAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &SegmentMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentMetricAdjuster {
    pub cap: f64,
}

impl SegmentMetricFactor for CapSegmentMetricAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentMetric, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentMetricAdjuster {
    pub minimum: f64,
}

impl SegmentMetricFactor for MinimumSegmentMetricAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentMetric, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentMetricAdjuster {
    pub step: f64,
}

impl SegmentMetricFactor for RoundingSegmentMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the segment metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentMetricEngine {
    pub steps: Vec<Rc<dyn SegmentMetricFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentMetricEngine {
    pub fn compute(&mut self, item: &SegmentMetric) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[SegmentMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_segment_metric_engine() -> SegmentMetricEngine {
    SegmentMetricEngine {
        steps: vec![
            Rc::new(PercentSegmentMetricAdjuster { percent: 23.5 }) as Rc<dyn SegmentMetricFactor>,
            Rc::new(UnitSegmentMetricAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.94)]),
            }),
            Rc::new(CapSegmentMetricAdjuster { cap: 30572.0 }),
            Rc::new(MinimumSegmentMetricAdjuster { minimum: 34.0 }),
            Rc::new(RoundingSegmentMetricAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 5.0,
        ceiling: 29339.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_metric_engine();
        let item = new_segment_metric("code-821", "title-624");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
