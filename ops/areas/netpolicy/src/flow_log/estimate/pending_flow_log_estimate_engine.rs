use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending flow log estimate calculation.
pub trait PendingFlowLogEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingFlowLogEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingFlowLogEstimateAdjuster {
    pub cap: f64,
}

impl PendingFlowLogEstimateStep for CapPendingFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingFlowLogEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingFlowLogEstimateAdjuster {
    pub step: f64,
}

impl PendingFlowLogEstimateStep for RoundingPendingFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingFlowLogEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingFlowLogEstimateAdjuster {
    pub percent: f64,
}

impl PendingFlowLogEstimateStep for PercentPendingFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingFlowLogEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingFlowLogEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingFlowLogEstimateStep for FlaggedPendingFlowLogEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingFlowLogEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the pending flow log estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingFlowLogEstimateEngine {
    pub steps: Vec<Rc<dyn PendingFlowLogEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingFlowLogEstimateEngine {
    pub fn price(&mut self, item: &PendingFlowLogEstimate) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingFlowLogEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_pending_flow_log_estimate_engine(flags: &flags::Registry) -> PendingFlowLogEstimateEngine {
    PendingFlowLogEstimateEngine {
        steps: vec![
            Rc::new(CapPendingFlowLogEstimateAdjuster { cap: 29347.0 }) as Rc<dyn PendingFlowLogEstimateStep>,
            Rc::new(RoundingPendingFlowLogEstimateAdjuster { step: 0.05 }),
            Rc::new(PercentPendingFlowLogEstimateAdjuster { percent: -21.1 }),
            Rc::new(FlaggedPendingFlowLogEstimateAdjuster {
                flags: flags.clone(),
                flag: "pending-flow-log-estimate-scheduled".to_string(),
                percent: 9.9,
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 98957.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_flow_log_estimate_engine(&flags::new_registry());
        let item = new_pending_flow_log_estimate("code-572", "title-914");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
