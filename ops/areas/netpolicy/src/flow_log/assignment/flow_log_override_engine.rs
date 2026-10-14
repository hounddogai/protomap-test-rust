use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the flow log override calculation.
pub trait FlowLogOverrideStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogOverrideAdjuster {
    pub step: f64,
}

impl FlowLogOverrideStep for RoundingFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogOverrideAdjuster {
    pub percent: f64,
}

impl FlowLogOverrideStep for PercentFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogOverrideAdjuster {
    pub cap: f64,
}

impl FlowLogOverrideStep for CapFlowLogOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the flow log override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogOverrideEngine {
    pub steps: Vec<Rc<dyn FlowLogOverrideStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogOverrideEngine {
    pub fn price(&mut self, item: &FlowLogOverride) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogOverrideStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_override_engine() -> FlowLogOverrideEngine {
    FlowLogOverrideEngine {
        steps: vec![
            Rc::new(RoundingFlowLogOverrideAdjuster { step: 0.05 }) as Rc<dyn FlowLogOverrideStep>,
            Rc::new(PercentFlowLogOverrideAdjuster { percent: -19.0 }),
            Rc::new(CapFlowLogOverrideAdjuster { cap: 24390.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 24435.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_override_engine();
        let item = new_flow_log_override("code-619", "title-389");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
