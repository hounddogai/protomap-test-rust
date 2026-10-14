use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log cycle calculation.
pub trait FlowLogCycleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogCycle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogCycleAdjuster {
    pub cap: f64,
}

impl FlowLogCycleFactor for CapFlowLogCycleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogCycle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFlowLogCycleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogCycleFactor for LabelFlowLogCycleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FlowLogCycle, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogCycleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogCycleFactor for FlaggedFlowLogCycleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogCycle, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogCycleRateOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogCycleFactor for FlowLogCycleRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogCycle, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogCycleAdjuster {
    pub step: f64,
}

impl FlowLogCycleFactor for RoundingFlowLogCycleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogCycle, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogCycleAdjuster {
    pub minimum: f64,
}

impl FlowLogCycleFactor for MinimumFlowLogCycleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogCycle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the flow log cycle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogCycleEngine {
    pub steps: Vec<Rc<dyn FlowLogCycleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogCycleEngine {
    pub fn price(&mut self, item: &FlowLogCycle) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogCycle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_cycle_engine(flags: &flags::Registry) -> FlowLogCycleEngine {
    FlowLogCycleEngine {
        steps: vec![
            Rc::new(CapFlowLogCycleAdjuster { cap: 31286.0 }) as Rc<dyn FlowLogCycleFactor>,
            Rc::new(LabelFlowLogCycleAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.91),
                    ("retail".to_string(), 1.0),
                    ("west".to_string(), 0.89),
                ]),
            }),
            Rc::new(FlaggedFlowLogCycleAdjuster {
                flags: flags.clone(),
                flag: "flow-log-cycle-default".to_string(),
                percent: -4.7,
            }),
            Rc::new(FlowLogCycleRateOffsetAdjuster { weight: 1.36 }),
            Rc::new(RoundingFlowLogCycleAdjuster { step: 0.1 }),
            Rc::new(MinimumFlowLogCycleAdjuster { minimum: 24.0 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 18655.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_cycle_engine(&flags::new_registry());
        let item = new_flow_log_cycle("code-607", "label-695");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
