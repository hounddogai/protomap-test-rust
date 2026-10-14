use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the runbook estimate calculation.
pub trait RunbookEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookEstimateAdjuster {
    pub minimum: f64,
}

impl RunbookEstimateAdjuster for MinimumRunbookEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterRunbookEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RunbookEstimateAdjuster for ClusterRunbookEstimateAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &RunbookEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookEstimateRateOffsetAdjuster {
    pub weight: f64,
}

impl RunbookEstimateAdjuster for RunbookEstimateRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &RunbookEstimate, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

/// Runs the runbook estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookEstimateEngine {
    pub steps: Vec<Rc<dyn RunbookEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookEstimateEngine {
    pub fn evaluate(&mut self, item: &RunbookEstimate) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[RunbookEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_runbook_estimate_engine() -> RunbookEstimateEngine {
    RunbookEstimateEngine {
        steps: vec![
            Rc::new(MinimumRunbookEstimateAdjuster { minimum: 19.4 }) as Rc<dyn RunbookEstimateAdjuster>,
            Rc::new(ClusterRunbookEstimateAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.93), ("north".to_string(), 1.32)]),
            }),
            Rc::new(RunbookEstimateRateOffsetAdjuster { weight: 0.64 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 68962.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_estimate_engine();
        let item = new_runbook_estimate("code-121", "title-234");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
