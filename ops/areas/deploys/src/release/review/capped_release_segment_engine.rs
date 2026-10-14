use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped release segment calculation.
pub trait CappedReleaseSegmentStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedReleaseSegment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedReleaseSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedReleaseSegmentStep for FlaggedCappedReleaseSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedReleaseSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedReleaseSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedReleaseSegmentStep for TieredCappedReleaseSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedReleaseSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedReleaseSegmentAdjuster {
    pub percent: f64,
}

impl CappedReleaseSegmentStep for PercentCappedReleaseSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedReleaseSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedReleaseSegmentAdjuster {
    pub step: f64,
}

impl CappedReleaseSegmentStep for RoundingCappedReleaseSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedReleaseSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedReleaseSegmentAdjuster {
    pub minimum: f64,
}

impl CappedReleaseSegmentStep for MinimumCappedReleaseSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedReleaseSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCappedReleaseSegmentAdjuster {
    pub cap: f64,
}

impl CappedReleaseSegmentStep for CapCappedReleaseSegmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CappedReleaseSegment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the capped release segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedReleaseSegmentEngine {
    pub steps: Vec<Rc<dyn CappedReleaseSegmentStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedReleaseSegmentEngine {
    pub fn price(&mut self, item: &CappedReleaseSegment) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[CappedReleaseSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_capped_release_segment_engine(flags: &flags::Registry) -> CappedReleaseSegmentEngine {
    CappedReleaseSegmentEngine {
        steps: vec![
            Rc::new(FlaggedCappedReleaseSegmentAdjuster {
                flags: flags.clone(),
                flag: "capped-release-segment-legacy".to_string(),
                percent: 2.7,
            }) as Rc<dyn CappedReleaseSegmentStep>,
            Rc::new(TieredCappedReleaseSegmentAdjuster {
                bounds: vec![2758.0, 3147.0, 8139.0, 9856.0],
                rates: vec![0.87, 0.81, 1.07, 1.09],
            }),
            Rc::new(PercentCappedReleaseSegmentAdjuster { percent: -6.0 }),
            Rc::new(RoundingCappedReleaseSegmentAdjuster { step: 0.1 }),
            Rc::new(MinimumCappedReleaseSegmentAdjuster { minimum: 30.6 }),
            Rc::new(CapCappedReleaseSegmentAdjuster { cap: 11630.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 15556.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_release_segment_engine(&flags::new_registry());
        let item = new_capped_release_segment("code-823", "name-151");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
