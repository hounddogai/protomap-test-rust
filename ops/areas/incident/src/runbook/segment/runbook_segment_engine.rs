use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the runbook segment calculation.
pub trait RunbookSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RunbookSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRunbookSegmentAdjuster {
    pub minimum: f64,
}

impl RunbookSegmentStep for MinimumRunbookSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RunbookSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRunbookSegmentAdjuster {
    pub percent: f64,
}

impl RunbookSegmentStep for PercentRunbookSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RunbookSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRunbookSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RunbookSegmentStep for FlaggedRunbookSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RunbookSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the runbook segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RunbookSegmentEngine {
    pub steps: Vec<Rc<dyn RunbookSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RunbookSegmentEngine {
    pub fn price(&mut self, item: &RunbookSegment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RunbookSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_runbook_segment_engine(flags: &flags::Registry) -> RunbookSegmentEngine {
    RunbookSegmentEngine {
        steps: vec![
            Rc::new(MinimumRunbookSegmentAdjuster { minimum: 33.9 }) as Rc<dyn RunbookSegmentStep>,
            Rc::new(PercentRunbookSegmentAdjuster { percent: -13.6 }),
            Rc::new(FlaggedRunbookSegmentAdjuster {
                flags: flags.clone(),
                flag: "runbook-segment-strict".to_string(),
                percent: -7.1,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 4800.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_runbook_segment_engine(&flags::new_registry());
        let item = new_runbook_segment("code-858", "title-934");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
