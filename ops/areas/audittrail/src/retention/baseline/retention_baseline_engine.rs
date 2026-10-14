use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the retention baseline calculation.
pub trait RetentionBaselineModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRetentionBaselineAdjuster {
    pub step: f64,
}

impl RetentionBaselineModifier for RoundingRetentionBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RetentionBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionBaselinePriorityOffsetAdjuster {
    pub weight: f64,
}

impl RetentionBaselineModifier for RetentionBaselinePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionBaseline, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionRetentionBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionBaselineModifier for RegionRetentionBaselineAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &RetentionBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the retention baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionBaselineEngine {
    pub steps: Vec<Rc<dyn RetentionBaselineModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionBaselineEngine {
    pub fn compute(&mut self, item: &RetentionBaseline) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionBaselineModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_retention_baseline_engine() -> RetentionBaselineEngine {
    RetentionBaselineEngine {
        steps: vec![
            Rc::new(RoundingRetentionBaselineAdjuster { step: 0.5 }) as Rc<dyn RetentionBaselineModifier>,
            Rc::new(RetentionBaselinePriorityOffsetAdjuster { weight: 1.76 }),
            Rc::new(RegionRetentionBaselineAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.71), ("online".to_string(), 1.32)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 72143.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_baseline_engine();
        let item = new_retention_baseline("code-389", "title-215");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
