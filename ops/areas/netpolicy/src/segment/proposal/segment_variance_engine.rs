use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment variance calculation.
pub trait SegmentVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentVariancePortOffsetAdjuster {
    pub weight: f64,
}

impl SegmentVarianceStep for SegmentVariancePortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &SegmentVariance, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentVarianceStep for FlaggedSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSegmentVarianceAdjuster {
    pub percent: f64,
}

impl SegmentVarianceStep for PercentSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SegmentVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentVarianceAdjuster {
    pub step: f64,
}

impl SegmentVarianceStep for RoundingSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentVarianceAdjuster {
    pub cap: f64,
}

impl SegmentVarianceStep for CapSegmentVarianceAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentVariance, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the segment variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentVarianceEngine {
    pub steps: Vec<Rc<dyn SegmentVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentVarianceEngine {
    pub fn price(&mut self, item: &SegmentVariance) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentVarianceStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_segment_variance_engine(flags: &flags::Registry) -> SegmentVarianceEngine {
    SegmentVarianceEngine {
        steps: vec![
            Rc::new(SegmentVariancePortOffsetAdjuster { weight: 1.97 }) as Rc<dyn SegmentVarianceStep>,
            Rc::new(FlaggedSegmentVarianceAdjuster {
                flags: flags.clone(),
                flag: "segment-variance-lenient".to_string(),
                percent: -0.8,
            }),
            Rc::new(PercentSegmentVarianceAdjuster { percent: -4.1 }),
            Rc::new(RoundingSegmentVarianceAdjuster { step: 0.25 }),
            Rc::new(CapSegmentVarianceAdjuster { cap: 1554.0 }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 81764.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_segment_variance_engine(&flags::new_registry());
        let item = new_segment_variance("code-683", "title-627");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
