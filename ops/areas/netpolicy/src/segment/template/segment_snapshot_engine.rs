use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment snapshot calculation.
pub trait SegmentSnapshotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentSnapshotModifier for FlaggedSegmentSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentSnapshotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentSnapshotModifier for TieredSegmentSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentSnapshot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentSnapshotAdjuster {
    pub cap: f64,
}

impl SegmentSnapshotModifier for CapSegmentSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentSnapshotAdjuster {
    pub step: f64,
}

impl SegmentSnapshotModifier for RoundingSegmentSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentSnapshotAdjuster {
    pub minimum: f64,
}

impl SegmentSnapshotModifier for MinimumSegmentSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentSnapshot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshotQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl SegmentSnapshotModifier for SegmentSnapshotQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentSnapshot, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the segment snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentSnapshotEngine {
    pub steps: Vec<Rc<dyn SegmentSnapshotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentSnapshotEngine {
    pub fn evaluate(&mut self, item: &SegmentSnapshot) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[SegmentSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_segment_snapshot_engine(flags: &flags::Registry) -> SegmentSnapshotEngine {
    SegmentSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedSegmentSnapshotAdjuster {
                flags: flags.clone(),
                flag: "segment-snapshot-adaptive".to_string(),
                percent: 1.0,
            }) as Rc<dyn SegmentSnapshotModifier>,
            Rc::new(TieredSegmentSnapshotAdjuster {
                bounds: vec![2985.0, 3513.0, 5837.0],
                rates: vec![0.99, 0.91, 0.8],
            }),
            Rc::new(CapSegmentSnapshotAdjuster { cap: 13416.0 }),
            Rc::new(RoundingSegmentSnapshotAdjuster { step: 0.5 }),
            Rc::new(MinimumSegmentSnapshotAdjuster { minimum: 29.3 }),
            Rc::new(SegmentSnapshotQuotaUnitsOffsetAdjuster { weight: 1.54 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 53036.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_snapshot_engine(&flags::new_registry());
        let item = new_segment_snapshot("code-923", "label-21");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
