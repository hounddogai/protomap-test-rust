use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment checkpoint calculation.
pub trait SegmentCheckpointAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceSegmentCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentCheckpointAdjuster for NamespaceSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &SegmentCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentCheckpointAdjuster {
    pub step: f64,
}

impl SegmentCheckpointAdjuster for RoundingSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentCheckpointAdjuster for FlaggedSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSegmentCheckpointAdjuster {
    pub minimum: f64,
}

impl SegmentCheckpointAdjuster for MinimumSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SegmentCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentCheckpointAdjuster {
    pub cap: f64,
}

impl SegmentCheckpointAdjuster for CapSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentCheckpointAdjuster for TieredSegmentCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the segment checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentCheckpointEngine {
    pub steps: Vec<Rc<dyn SegmentCheckpointAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentCheckpointEngine {
    pub fn compute(&mut self, item: &SegmentCheckpoint) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[SegmentCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_segment_checkpoint_engine(flags: &flags::Registry) -> SegmentCheckpointEngine {
    SegmentCheckpointEngine {
        steps: vec![
            Rc::new(NamespaceSegmentCheckpointAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.11)]),
            }) as Rc<dyn SegmentCheckpointAdjuster>,
            Rc::new(RoundingSegmentCheckpointAdjuster { step: 0.25 }),
            Rc::new(FlaggedSegmentCheckpointAdjuster {
                flags: flags.clone(),
                flag: "segment-checkpoint-lenient".to_string(),
                percent: -8.1,
            }),
            Rc::new(MinimumSegmentCheckpointAdjuster { minimum: 18.8 }),
            Rc::new(CapSegmentCheckpointAdjuster { cap: 2322.0 }),
            Rc::new(TieredSegmentCheckpointAdjuster {
                bounds: vec![5356.0, 9786.0],
                rates: vec![1.0, 0.84],
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 25417.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_checkpoint_engine(&flags::new_registry());
        let item = new_segment_checkpoint("code-407", "name-105");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
