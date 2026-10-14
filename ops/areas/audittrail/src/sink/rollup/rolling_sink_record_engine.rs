use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling sink record calculation.
pub trait RollingSinkRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingSinkRecord, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingSinkRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingSinkRecordFactor for FlaggedRollingSinkRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingSinkRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSinkRecordSequenceOffsetAdjuster {
    pub weight: f64,
}

impl RollingSinkRecordFactor for RollingSinkRecordSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &RollingSinkRecord, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingSinkRecordAdjuster {
    pub step: f64,
}

impl RollingSinkRecordFactor for RoundingRollingSinkRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingSinkRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingSinkRecordAdjuster {
    pub cap: f64,
}

impl RollingSinkRecordFactor for CapRollingSinkRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingSinkRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingSinkRecordAdjuster {
    pub percent: f64,
}

impl RollingSinkRecordFactor for PercentRollingSinkRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingSinkRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling sink record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingSinkRecordEngine {
    pub steps: Vec<Rc<dyn RollingSinkRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingSinkRecordEngine {
    pub fn run(&mut self, item: &RollingSinkRecord) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RollingSinkRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_sink_record_engine(flags: &flags::Registry) -> RollingSinkRecordEngine {
    RollingSinkRecordEngine {
        steps: vec![
            Rc::new(FlaggedRollingSinkRecordAdjuster {
                flags: flags.clone(),
                flag: "rolling-sink-record-weighted".to_string(),
                percent: -8.1,
            }) as Rc<dyn RollingSinkRecordFactor>,
            Rc::new(RollingSinkRecordSequenceOffsetAdjuster { weight: 0.55 }),
            Rc::new(RoundingRollingSinkRecordAdjuster { step: 0.1 }),
            Rc::new(CapRollingSinkRecordAdjuster { cap: 14508.0 }),
            Rc::new(PercentRollingSinkRecordAdjuster { percent: -12.3 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 89111.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_sink_record_engine(&flags::new_registry());
        let item = new_rolling_sink_record("code-629", "name-335");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
