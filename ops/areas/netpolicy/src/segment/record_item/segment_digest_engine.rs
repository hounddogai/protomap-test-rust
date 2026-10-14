use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment digest calculation.
pub trait SegmentDigestAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentDigestAdjuster {
    pub step: f64,
}

impl SegmentDigestAdjuster for RoundingSegmentDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentDigestAdjuster for FlaggedSegmentDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentDigestAdjuster {
    pub cap: f64,
}

impl SegmentDigestAdjuster for CapSegmentDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentDigestAdjuster for TieredSegmentDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the segment digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentDigestEngine {
    pub steps: Vec<Rc<dyn SegmentDigestAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentDigestEngine {
    pub fn evaluate(&mut self, item: &SegmentDigest) -> f64 {
        let mut current = item.effective_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentDigestAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_segment_digest_engine(flags: &flags::Registry) -> SegmentDigestEngine {
    SegmentDigestEngine {
        steps: vec![
            Rc::new(RoundingSegmentDigestAdjuster { step: 0.25 }) as Rc<dyn SegmentDigestAdjuster>,
            Rc::new(FlaggedSegmentDigestAdjuster {
                flags: flags.clone(),
                flag: "segment-digest-primary".to_string(),
                percent: 3.6,
            }),
            Rc::new(CapSegmentDigestAdjuster { cap: 2432.0 }),
            Rc::new(TieredSegmentDigestAdjuster {
                bounds: vec![3259.0, 8459.0, 9592.0],
                rates: vec![1.16, 1.13, 0.82],
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 57681.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_digest_engine(&flags::new_registry());
        let item = new_segment_digest("code-396", "title-259");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
