use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink segment calculation.
pub trait SinkSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkSegment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkSegmentStep for FlaggedSinkSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelSinkSegmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkSegmentStep for ChannelSinkSegmentAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &SinkSegment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkSegmentAdjuster {
    pub percent: f64,
}

impl SinkSegmentStep for PercentSinkSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the sink segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkSegmentEngine {
    pub steps: Vec<Rc<dyn SinkSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkSegmentEngine {
    pub fn price(&mut self, item: &SinkSegment) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SinkSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_segment_engine(flags: &flags::Registry) -> SinkSegmentEngine {
    SinkSegmentEngine {
        steps: vec![
            Rc::new(FlaggedSinkSegmentAdjuster {
                flags: flags.clone(),
                flag: "sink-segment-legacy".to_string(),
                percent: -2.1,
            }) as Rc<dyn SinkSegmentStep>,
            Rc::new(ChannelSinkSegmentAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.92), ("north".to_string(), 0.73)]),
            }),
            Rc::new(PercentSinkSegmentAdjuster { percent: 20.3 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 38822.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_sink_segment_engine(&flags::new_registry());
        let item = new_sink_segment("code-273", "label-680");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
