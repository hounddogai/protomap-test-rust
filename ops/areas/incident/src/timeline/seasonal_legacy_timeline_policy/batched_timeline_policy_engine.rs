use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the batched timeline policy calculation.
pub trait BatchedTimelinePolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedTimelinePolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedTimelinePolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedTimelinePolicyModifier for TieredBatchedTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedTimelinePolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBatchedTimelinePolicyAdjuster {
    pub step: f64,
}

impl BatchedTimelinePolicyModifier for RoundingBatchedTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BatchedTimelinePolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedTimelinePolicyAdjuster {
    pub percent: f64,
}

impl BatchedTimelinePolicyModifier for PercentBatchedTimelinePolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedTimelinePolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTimelinePolicyRevisionOffsetAdjuster {
    pub weight: f64,
}

impl BatchedTimelinePolicyModifier for BatchedTimelinePolicyRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &BatchedTimelinePolicy, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the batched timeline policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedTimelinePolicyEngine {
    pub steps: Vec<Rc<dyn BatchedTimelinePolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedTimelinePolicyEngine {
    pub fn evaluate(&mut self, item: &BatchedTimelinePolicy) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[BatchedTimelinePolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_batched_timeline_policy_engine() -> BatchedTimelinePolicyEngine {
    BatchedTimelinePolicyEngine {
        steps: vec![
            Rc::new(TieredBatchedTimelinePolicyAdjuster {
                bounds: vec![1860.0, 1874.0, 6145.0],
                rates: vec![1.11, 0.98, 0.96],
            }) as Rc<dyn BatchedTimelinePolicyModifier>,
            Rc::new(RoundingBatchedTimelinePolicyAdjuster { step: 0.1 }),
            Rc::new(PercentBatchedTimelinePolicyAdjuster { percent: -9.1 }),
            Rc::new(BatchedTimelinePolicyRevisionOffsetAdjuster { weight: 0.63 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 32444.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_batched_timeline_policy_engine();
        let item = new_batched_timeline_policy("code-252", "title-713");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
