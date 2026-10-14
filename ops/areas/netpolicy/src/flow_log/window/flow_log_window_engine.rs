use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log window calculation.
pub trait FlowLogWindowFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogWindowAdjuster {
    pub percent: f64,
}

impl FlowLogWindowFactor for PercentFlowLogWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogWindowGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogWindowFactor for FlowLogWindowGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogWindow, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogWindowFactor for FlaggedFlowLogWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogWindowAdjuster {
    pub minimum: f64,
}

impl FlowLogWindowFactor for MinimumFlowLogWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogWindowAdjuster {
    pub cap: f64,
}

impl FlowLogWindowFactor for CapFlowLogWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the flow log window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogWindowEngine {
    pub steps: Vec<Rc<dyn FlowLogWindowFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogWindowEngine {
    pub fn price(&mut self, item: &FlowLogWindow) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_window_engine(flags: &flags::Registry) -> FlowLogWindowEngine {
    FlowLogWindowEngine {
        steps: vec![
            Rc::new(PercentFlowLogWindowAdjuster { percent: 16.8 }) as Rc<dyn FlowLogWindowFactor>,
            Rc::new(FlowLogWindowGraceDaysOffsetAdjuster { weight: 0.8 }),
            Rc::new(FlaggedFlowLogWindowAdjuster {
                flags: flags.clone(),
                flag: "flow-log-window-regional".to_string(),
                percent: 2.3,
            }),
            Rc::new(MinimumFlowLogWindowAdjuster { minimum: 12.7 }),
            Rc::new(CapFlowLogWindowAdjuster { cap: 36761.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 68475.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_window_engine(&flags::new_registry());
        let item = new_flow_log_window("code-335", "name-538");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
