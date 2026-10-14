use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual segment entry calculation.
pub trait ManualSegmentEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualSegmentEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualSegmentEntryAdjuster {
    pub percent: f64,
}

impl ManualSegmentEntryModifier for PercentManualSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualSegmentEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualSegmentEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualSegmentEntryModifier for FlaggedManualSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualSegmentEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualSegmentEntryAdjuster {
    pub cap: f64,
}

impl ManualSegmentEntryModifier for CapManualSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualSegmentEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualSegmentEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualSegmentEntryModifier for TieredManualSegmentEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualSegmentEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual segment entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualSegmentEntryEngine {
    pub steps: Vec<Rc<dyn ManualSegmentEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualSegmentEntryEngine {
    pub fn run(&mut self, item: &ManualSegmentEntry) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualSegmentEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualSegmentEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_segment_entry_engine(flags: &flags::Registry) -> ManualSegmentEntryEngine {
    ManualSegmentEntryEngine {
        steps: vec![
            Rc::new(PercentManualSegmentEntryAdjuster { percent: -11.0 }) as Rc<dyn ManualSegmentEntryModifier>,
            Rc::new(FlaggedManualSegmentEntryAdjuster {
                flags: flags.clone(),
                flag: "manual-segment-entry-regional".to_string(),
                percent: 8.4,
            }),
            Rc::new(CapManualSegmentEntryAdjuster { cap: 5377.0 }),
            Rc::new(TieredManualSegmentEntryAdjuster {
                bounds: vec![2110.0, 6969.0],
                rates: vec![0.83, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 94117.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_segment_entry_engine(&flags::new_registry());
        let item = new_manual_segment_entry("code-186", "name-224");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
