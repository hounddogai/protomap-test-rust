use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the retention adjustment calculation.
pub trait RetentionAdjustmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RetentionAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAdjustmentStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl RetentionAdjustmentAdjuster for RetentionAdjustmentStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &RetentionAdjustment, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRetentionAdjustmentAdjuster {
    pub percent: f64,
}

impl RetentionAdjustmentAdjuster for PercentRetentionAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RetentionAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRetentionAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RetentionAdjustmentAdjuster for ClusterRetentionAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RetentionAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the retention adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RetentionAdjustmentEngine {
    pub steps: Vec<Rc<dyn RetentionAdjustmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RetentionAdjustmentEngine {
    pub fn evaluate(&mut self, item: &RetentionAdjustment) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RetentionAdjustmentAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RetentionAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_retention_adjustment_engine() -> RetentionAdjustmentEngine {
    RetentionAdjustmentEngine {
        steps: vec![
            Rc::new(RetentionAdjustmentStartsAtOffsetAdjuster { weight: 1.75 }) as Rc<dyn RetentionAdjustmentAdjuster>,
            Rc::new(PercentRetentionAdjustmentAdjuster { percent: -8.7 }),
            Rc::new(ClusterRetentionAdjustmentAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.91),
                    ("central".to_string(), 0.93),
                    ("west".to_string(), 1.04),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 18639.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_retention_adjustment_engine();
        let item = new_retention_adjustment("code-956", "title-82");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
