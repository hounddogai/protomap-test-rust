use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the flow log baseline calculation.
pub trait FlowLogBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogBaselineAdjuster {
    pub cap: f64,
}

impl FlowLogBaselineStep for CapFlowLogBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogBaselineAdjuster {
    pub step: f64,
}

impl FlowLogBaselineStep for RoundingFlowLogBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameFlowLogBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogBaselineStep for NameFlowLogBaselineAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &FlowLogBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the flow log baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogBaselineEngine {
    pub steps: Vec<Rc<dyn FlowLogBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogBaselineEngine {
    pub fn compute(&mut self, item: &FlowLogBaseline) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_flow_log_baseline_engine() -> FlowLogBaselineEngine {
    FlowLogBaselineEngine {
        steps: vec![
            Rc::new(CapFlowLogBaselineAdjuster { cap: 24724.0 }) as Rc<dyn FlowLogBaselineStep>,
            Rc::new(RoundingFlowLogBaselineAdjuster { step: 1.0 }),
            Rc::new(NameFlowLogBaselineAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.17), ("south".to_string(), 0.86)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 84598.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_baseline_engine();
        let item = new_flow_log_baseline("code-458", "name-927");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
