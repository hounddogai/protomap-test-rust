use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the segment rollup calculation.
pub trait SegmentRollupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentRollupAdjuster {
    pub percent: f64,
}

impl SegmentRollupFactor for PercentSegmentRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentRollupAdjuster {
    pub step: f64,
}

impl SegmentRollupFactor for RoundingSegmentRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentRollupAdjuster {
    pub minimum: f64,
}

impl SegmentRollupFactor for MinimumSegmentRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the segment rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentRollupEngine {
    pub steps: Vec<Rc<dyn SegmentRollupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentRollupEngine {
    pub fn compute(&mut self, item: &SegmentRollup) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentRollupFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_segment_rollup_engine() -> SegmentRollupEngine {
    SegmentRollupEngine {
        steps: vec![
            Rc::new(PercentSegmentRollupAdjuster { percent: -8.9 }) as Rc<dyn SegmentRollupFactor>,
            Rc::new(RoundingSegmentRollupAdjuster { step: 0.1 }),
            Rc::new(MinimumSegmentRollupAdjuster { minimum: 17.9 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 19501.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_rollup_engine();
        let item = new_segment_rollup("code-229", "title-993");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
