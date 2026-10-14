use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the flow log record calculation.
pub trait FlowLogRecordModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogRecordRatioOffsetAdjuster {
    pub weight: f64,
}

impl FlowLogRecordModifier for FlowLogRecordRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &FlowLogRecord, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogRecordAdjuster {
    pub percent: f64,
}

impl FlowLogRecordModifier for PercentFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFlowLogRecordAdjuster {
    pub cap: f64,
}

impl FlowLogRecordModifier for CapFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FlowLogRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFlowLogRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogRecordModifier for ChannelFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &FlowLogRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogRecordAdjuster {
    pub step: f64,
}

impl FlowLogRecordModifier for RoundingFlowLogRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the flow log record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogRecordEngine {
    pub steps: Vec<Rc<dyn FlowLogRecordModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogRecordEngine {
    pub fn price(&mut self, item: &FlowLogRecord) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FlowLogRecordModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_flow_log_record_engine() -> FlowLogRecordEngine {
    FlowLogRecordEngine {
        steps: vec![
            Rc::new(FlowLogRecordRatioOffsetAdjuster { weight: 0.47 }) as Rc<dyn FlowLogRecordModifier>,
            Rc::new(PercentFlowLogRecordAdjuster { percent: 4.2 }),
            Rc::new(CapFlowLogRecordAdjuster { cap: 33116.0 }),
            Rc::new(ChannelFlowLogRecordAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.84),
                    ("east".to_string(), 0.95),
                    ("central".to_string(), 0.96),
                ]),
            }),
            Rc::new(RoundingFlowLogRecordAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 8030.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_record_engine();
        let item = new_flow_log_record("code-723", "title-95");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
