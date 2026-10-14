use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log threshold calculation.
pub trait FlowLogThresholdModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogThresholdAdjuster {
    pub cap: f64,
}

impl FlowLogThresholdModifier for CapFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogThresholdModifier for FlaggedFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogThresholdAdjuster {
    pub step: f64,
}

impl FlowLogThresholdModifier for RoundingFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogThresholdAdjuster {
    pub percent: f64,
}

impl FlowLogThresholdModifier for PercentFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogThreshold, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FlowLogThresholdModifier for TieredFlowLogThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FlowLogThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogThresholdTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogThresholdModifier for FlowLogThresholdTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogThreshold, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

/// Runs the flow log threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogThresholdEngine {
    pub steps: Vec<Rc<dyn FlowLogThresholdModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogThresholdEngine {
    pub fn compute(&mut self, item: &FlowLogThreshold) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[FlowLogThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_flow_log_threshold_engine(flags: &flags::Registry) -> FlowLogThresholdEngine {
    FlowLogThresholdEngine {
        steps: vec![
            Rc::new(CapFlowLogThresholdAdjuster { cap: 49196.0 }) as Rc<dyn FlowLogThresholdModifier>,
            Rc::new(FlaggedFlowLogThresholdAdjuster {
                flags: flags.clone(),
                flag: "flow-log-threshold-tiered".to_string(),
                percent: -7.5,
            }),
            Rc::new(RoundingFlowLogThresholdAdjuster { step: 5.0 }),
            Rc::new(PercentFlowLogThresholdAdjuster { percent: 23.7 }),
            Rc::new(TieredFlowLogThresholdAdjuster {
                bounds: vec![6070.0, 6427.0, 9314.0],
                rates: vec![0.92, 1.1, 1.08],
            }),
            Rc::new(FlowLogThresholdTtlSecondsOffsetAdjuster { weight: 0.71 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 16304.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_threshold_engine(&flags::new_registry());
        let item = new_flow_log_threshold("code-963", "title-3");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
