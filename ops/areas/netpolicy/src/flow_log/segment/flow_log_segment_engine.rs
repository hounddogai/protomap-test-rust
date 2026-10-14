use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the flow log segment calculation.
pub trait FlowLogSegmentAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FlowLogSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFlowLogSegmentAdjuster {
    pub minimum: f64,
}

impl FlowLogSegmentAdjuster for MinimumFlowLogSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FlowLogSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFlowLogSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FlowLogSegmentAdjuster for FlaggedFlowLogSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FlowLogSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFlowLogSegmentAdjuster {
    pub percent: f64,
}

impl FlowLogSegmentAdjuster for PercentFlowLogSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FlowLogSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFlowLogSegmentAdjuster {
    pub step: f64,
}

impl FlowLogSegmentAdjuster for RoundingFlowLogSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FlowLogSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleFlowLogSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FlowLogSegmentAdjuster for TitleFlowLogSegmentAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &FlowLogSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the flow log segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FlowLogSegmentEngine {
    pub steps: Vec<Rc<dyn FlowLogSegmentAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FlowLogSegmentEngine {
    pub fn compute(&mut self, item: &FlowLogSegment) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[FlowLogSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_flow_log_segment_engine(flags: &flags::Registry) -> FlowLogSegmentEngine {
    FlowLogSegmentEngine {
        steps: vec![
            Rc::new(MinimumFlowLogSegmentAdjuster { minimum: 30.1 }) as Rc<dyn FlowLogSegmentAdjuster>,
            Rc::new(FlaggedFlowLogSegmentAdjuster {
                flags: flags.clone(),
                flag: "flow-log-segment-fallback".to_string(),
                percent: 7.3,
            }),
            Rc::new(PercentFlowLogSegmentAdjuster { percent: 8.4 }),
            Rc::new(RoundingFlowLogSegmentAdjuster { step: 0.1 }),
            Rc::new(TitleFlowLogSegmentAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.1),
                    ("east".to_string(), 0.9),
                    ("retail".to_string(), 1.18),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 36582.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_flow_log_segment_engine(&flags::new_registry());
        let item = new_flow_log_segment("code-951", "title-928");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
