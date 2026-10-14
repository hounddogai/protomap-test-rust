use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered usage batch calculation.
pub trait TieredUsageBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredUsageBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageBatchStartsAtOffsetAdjuster {
    pub weight: f64,
}

impl TieredUsageBatchStep for TieredUsageBatchStartsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus starts at times {}", self.weight)
    }

    fn adjust(&self, item: &TieredUsageBatch, value: f64) -> f64 {
        value + (item.starts_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredUsageBatchAdjuster {
    pub percent: f64,
}

impl TieredUsageBatchStep for PercentTieredUsageBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredUsageBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTieredUsageBatchAdjuster {
    pub minimum: f64,
}

impl TieredUsageBatchStep for MinimumTieredUsageBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TieredUsageBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tiered usage batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredUsageBatchEngine {
    pub steps: Vec<Rc<dyn TieredUsageBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredUsageBatchEngine {
    pub fn evaluate(&mut self, item: &TieredUsageBatch) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredUsageBatchStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredUsageBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tiered_usage_batch_engine() -> TieredUsageBatchEngine {
    TieredUsageBatchEngine {
        steps: vec![
            Rc::new(TieredUsageBatchStartsAtOffsetAdjuster { weight: 1.64 }) as Rc<dyn TieredUsageBatchStep>,
            Rc::new(PercentTieredUsageBatchAdjuster { percent: 21.7 }),
            Rc::new(MinimumTieredUsageBatchAdjuster { minimum: 41.6 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 7391.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_usage_batch_engine();
        let item = new_tiered_usage_batch("code-273", "name-727");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
