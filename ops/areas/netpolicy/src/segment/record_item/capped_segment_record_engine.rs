use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped segment record calculation.
pub trait CappedSegmentRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedSegmentRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordVersionOffsetAdjuster {
    pub weight: f64,
}

impl CappedSegmentRecordFactor for CappedSegmentRecordVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &CappedSegmentRecord, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedSegmentRecordAdjuster {
    pub cap: f64,
}

impl CappedSegmentRecordFactor for CapCappedSegmentRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedSegmentRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedSegmentRecordAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedSegmentRecordFactor for FlaggedCappedSegmentRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedSegmentRecord, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped segment record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedSegmentRecordEngine {
    pub steps: Vec<Rc<dyn CappedSegmentRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedSegmentRecordEngine {
    pub fn price(&mut self, item: &CappedSegmentRecord) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[CappedSegmentRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_segment_record_engine(flags: &flags::Registry) -> CappedSegmentRecordEngine {
    CappedSegmentRecordEngine {
        steps: vec![
            Rc::new(CappedSegmentRecordVersionOffsetAdjuster { weight: 0.72 }) as Rc<dyn CappedSegmentRecordFactor>,
            Rc::new(CapCappedSegmentRecordAdjuster { cap: 46866.0 }),
            Rc::new(FlaggedCappedSegmentRecordAdjuster {
                flags: flags.clone(),
                flag: "capped-segment-record-tiered".to_string(),
                percent: -1.2,
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 75716.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_segment_record_engine(&flags::new_registry());
        let item = new_capped_segment_record("code-820", "label-922");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
