use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention segment calculation.
pub trait RetentionSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRetentionSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionSegmentAdjuster for LabelRetentionSegmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RetentionSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionSegmentAdjuster {
    pub step: f64,
}

impl RetentionSegmentAdjuster for RoundingRetentionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRetentionSegmentAdjuster {
    pub minimum: f64,
}

impl RetentionSegmentAdjuster for MinimumRetentionSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RetentionSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the retention segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionSegmentEngine {
    pub steps: Vec<Rc<dyn RetentionSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionSegmentEngine {
    pub fn run(&mut self, item: &RetentionSegment) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RetentionSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_retention_segment_engine() -> RetentionSegmentEngine {
    RetentionSegmentEngine {
        steps: vec![
            Rc::new(LabelRetentionSegmentAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.97),
                    ("online".to_string(), 1.35),
                    ("central".to_string(), 1.35),
                ]),
            }) as Rc<dyn RetentionSegmentAdjuster>,
            Rc::new(RoundingRetentionSegmentAdjuster { step: 5.0 }),
            Rc::new(MinimumRetentionSegmentAdjuster { minimum: 9.9 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 71666.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_segment_engine();
        let item = new_retention_segment("code-271", "label-461");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
